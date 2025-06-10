{
  description = "Template for Holochain app development";

  inputs = {
    holonix.url = "github:holochain/holonix/main-0.5";

    nixpkgs.follows = "holonix/nixpkgs";
    flake-parts.follows = "holonix/flake-parts";
    rust-overlay.follows = "holonix/rust-overlay";

    scaffolding.url = "github:darksoil-studio/scaffolding/main-0.5";
    holochain-nix-builders.url =
      "github:darksoil-studio/holochain-nix-builders/main-0.5";
    tauri-plugin-holochain.url =
      "github:darksoil-studio/tauri-plugin-holochain/main-0.5";
    playground.url = "github:darksoil-studio/holochain-playground/main-0.5";

    # notifications-zome.url =
    #   "github:darksoil-studio/notifications-zome/main-0.5";
    linked-devices-zome.url =
      "github:darksoil-studio/linked-devices-zome/main-0.5";
    friends-zome.url = "github:darksoil-studio/friends-zome/main-0.5";
    messenger-zome.url = "github:darksoil-studio/messenger-zome/main-0.5";

    service-providers.url = "github:darksoil-studio/service-providers/main-0.5";

    push-notifications-service.url =
      "github:darksoil-studio/push-notifications-service/main-0.5";

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
      imports = [
        ./happ.nix
        ./tauri-app.nix
        ./aon/raspberry-pi.nix
        inputs.holochain-nix-builders.outputs.flakeModules.builders
      ];

      systems = builtins.attrNames inputs.holonix.devShells;
      perSystem = { inputs', config, pkgs, system, ... }: rec {
        devShells.default = pkgs.mkShell {
          inputsFrom = [
            inputs'.tauri-plugin-holochain.devShells.holochainTauriDev
            inputs'.scaffolding.devShells.synchronized-pnpm
            inputs'.holonix.devShells.default
          ];
          packages = [
            inputs'.holochain-nix-builders.packages.holochain
            inputs'.tauri-plugin-holochain.packages.hc-pilot
            inputs'.scaffolding.packages.hc-scaffold-happ
            inputs'.playground.packages.hc-playground
            inputs'.push-notifications-service.packages.test-push-notifications-service
          ];
        };

        devShells.androidDev = pkgs.mkShell {
          inputsFrom = [
            inputs'.tauri-plugin-holochain.devShells.holochainTauriAndroidDev
            devShells.default
          ];
          shellHook = ''
            export CARGO_TARGET_DIR=$(pwd)/target/android
          '';
        };
      };
    };
}
