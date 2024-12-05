{
  description = "Template for Holochain app development";

  inputs = {
    holonix.url = "github:holochain/holonix/main-0.4";

    nixpkgs.follows = "holonix/nixpkgs";
    flake-parts.follows = "holonix/flake-parts";

    tnesh-stack.url = "github:darksoil-studio/tnesh-stack/main-0.4";
    p2p-shipyard.url = "github:darksoil-studio/p2p-shipyard/wamr";
    playground.url = "github:darksoil-studio/holochain-playground/main-0.4";

    file-storage.url = "github:darksoil-studio/file-storage/main-0.4";
    linked-devices-zome.url =
      "github:darksoil-studio/linked-devices-zome/main-0.4";
    profiles-zome.url = "github:darksoil-studio/profiles-zome/main-0.4";
    messenger-zome.url =
      "git+ssh://git@github.com/darksoil-studio/messenger-zome?ref=main-0.4";
    aons.url =
      "git+ssh://git@github.com/darksoil-studio/always-online-nodes?ref=main";

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
      perSystem = { inputs', config, pkgs, system, ... }: {
        devShells.default = pkgs.mkShell {
          inputsFrom = [
            inputs'.p2p-shipyard.devShells.holochainTauriDev
            inputs'.tnesh-stack.devShells.synchronized-pnpm
            inputs'.holonix.devShells.default
          ];
          packages = [
            (inputs'.holonix.packages.holochain.override {
              cargoExtraArgs = " --features unstable-functions";
            })
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
            (inputs'.holonix.packages.holochain.override {
              cargoExtraArgs = " --features unstable-functions";
            })
            inputs'.tnesh-stack.packages.hc-scaffold-happ
            inputs'.playground.packages.hc-playground
          ];
        };
      };
    };
}
