{
  description = "Template for Holochain app development";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.05";
    tauri-plugin-holochain.url =
      "github:darksoil-studio/tauri-plugin-holochain";
    tauri-plugin-holochain.inputs.webkitnixpkgs.follows = "nixpkgs";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-parts.url = "github:hercules-ci/flake-parts";

    garnix-lib = {
      url = "github:garnix-io/garnix-lib";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nixos-compose.url = "github:garnix-io/nixos-compose";

    nixos-generators.url = "github:nix-community/nixos-generators";
  };

  nixConfig = {
    extra-substituters = [
      "https://holochain-ci.cachix.org"
      "https://darksoil-studio.cachix.org"
    ];
    extra-trusted-public-keys = [
      "holochain-ci.cachix.org-1:5IUSkZc0aoRS53rfkvH9Kid40NpyjwCMCzwRTXy+QN8="
      "darksoil-studio.cachix.org-1:UEi+aujy44s41XL/pscLw37KEVpTEIn8N/kn7jO8rkc="
    ];
  };

  outputs = inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [
        ./nix/servers.nix
        ./nix/tauri-app.nix
        ./nix/raspberry-pi.nix
        # inputs.p2p-shipyard.outputs.flakeModules.builders
      ];

      systems =
        [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin" ];
      perSystem = { inputs', config, pkgs, system, ... }:
        let
          overlays = [ (import inputs.rust-overlay) ];
          pkgs = import inputs.nixpkgs { inherit system overlays; };

          rust = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        in rec {
          devShells.default = pkgs.mkShell {
            inputsFrom =
              [ inputs'.tauri-plugin-holochain.devShells.holochainTauriDev ];
            packages = [
              pkgs.mprocs
              pkgs.pnpm
              rust
              inputs'.nixos-compose.packages.default
            ];
          };

          devShells.androidDev = pkgs.mkShell {
            inputsFrom = [
              devShells.default
              inputs'.tauri-plugin-holochain.devShells.holochainTauriAndroidDev
            ];
          };
        };
    };
}
