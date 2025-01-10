{
  description = "Template for Holochain app development";

  inputs = {
    holonix.url = "github:holochain/holonix/main-0.4";

    nixpkgs.follows = "holonix/nixpkgs";
    flake-parts.follows = "holonix/flake-parts";
    rust-overlay.follows = "holonix/rust-overlay";

    tnesh-stack.url = "github:darksoil-studio/tnesh-stack/main-0.4";
    p2p-shipyard.url = "github:darksoil-studio/p2p-shipyard/main-0.4";
    playground.url = "github:darksoil-studio/holochain-playground/main-0.4";

    file-storage.follows = "messenger-zome/file-storage";
    linked-devices-zome.follows = "messenger-zome/linked-devices-zome";
    profiles-zome.follows = "messenger-zome/profiles-zome";
    messenger-zome.url = "github:darksoil-studio/messenger-zome/utils";
    aons.url = "github:darksoil-studio/always-online-nodes/main";

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
      imports = [ ./happ.nix ./tauri-app.nix ./aon/raspberry-pi.nix ];

      systems = builtins.attrNames inputs.holonix.devShells;
      perSystem = { inputs', config, pkgs, system, ... }:

        let
          overlays = [ (import inputs.rust-overlay) ];
          pkgs = import inputs.nixpkgs { inherit system overlays; };

          rustVersion = "1.81.0";

          # define Rust toolchain version and targets to be used in this flake
          rust = (pkgs.rust-bin.stable.${rustVersion}.minimal.override {
            extensions = [ "clippy" "rustfmt" ];
            targets = [ "wasm32-unknown-unknown" ];
          });

        in {
          devShells.pnpm = pkgs.mkShell {
            inputsFrom = [ inputs'.tnesh-stack.devShells.synchronized-pnpm ];
          };
          devShells.default = pkgs.mkShell {
            inputsFrom = [
              inputs'.p2p-shipyard.devShells.holochainTauriDev
              inputs'.tnesh-stack.devShells.synchronized-pnpm
              inputs'.holonix.devShells.default
            ];
            packages = [
              (rust)
              inputs'.tnesh-stack.packages.holochain
              inputs'.p2p-shipyard.packages.hc-pilot
              inputs'.tnesh-stack.packages.hc-scaffold-happ
              inputs'.playground.packages.hc-playground
            ];
          };
          devShells.androidDev = pkgs.mkShell {
            inputsFrom = [
              inputs'.p2p-shipyard.devShells.holochainTauriAndroidDev
              inputs'.tnesh-stack.devShells.synchronized-pnpm
              inputs'.holonix.devShells.default
            ];
            packages = [
              rust
              inputs'.tnesh-stack.packages.holochain
              inputs'.tnesh-stack.packages.hc-scaffold-happ
              inputs'.playground.packages.hc-playground
            ];
          };
        };
    };
}
