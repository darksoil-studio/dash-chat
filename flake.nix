{
  description = "Template for Holochain app development";

  inputs = {
    p2p-shipyard.url = "github:darksoil-studio/p2p-shipyard/main-0.6";
    nixpkgs.follows = "p2p-shipyard/nixpkgs";

    garnix-lib = {
      url = "github:garnix-io/garnix-lib";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    nixos-generators.url = "github:nix-community/nixos-generators";
  };

  nixConfig = {
    extra-substituters = [
      "https://holochain-ci.cachix.org"
      "https://darksoil-studio.cachix.org"
      "https://cache.garnix.io"
    ];
    extra-trusted-public-keys = [
      "holochain-ci.cachix.org-1:5IUSkZc0aoRS53rfkvH9Kid40NpyjwCMCzwRTXy+QN8="
      "darksoil-studio.cachix.org-1:UEi+aujy44s41XL/pscLw37KEVpTEIn8N/kn7jO8rkc="
      "cache.garnix.io:CTFPyKSLcx5RMJKfLo5EEPUObbA78b0YQ2DTCJXqr9g="
    ];
  };

  outputs = inputs:
    inputs.p2p-shipyard.inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [
        ./servers.nix
        ./happ.nix
        ./tauri-app.nix
        ./aon/raspberry-pi.nix
        inputs.p2p-shipyard.outputs.flakeModules.builders
      ];

      systems = builtins.attrNames inputs.p2p-shipyard.devShells;
      perSystem = { inputs', config, pkgs, system, ... }: rec {
        devShells.default = pkgs.mkShell {
          inputsFrom = [
            inputs'.p2p-shipyard.devShells.holochainTauriDev
            inputs'.p2p-shipyard.devShells.synchronized-pnpm
            inputs'.p2p-shipyard.devShells.default
          ];
          packages = [
            inputs'.p2p-shipyard.packages.holochain
            inputs'.p2p-shipyard.packages.hc-pilot
            inputs'.p2p-shipyard.packages.hc-scaffold-happ
            inputs'.p2p-shipyard.packages.hc-playground
            inputs'.p2p-shipyard.packages.test-push-notifications-service
            inputs'.p2p-shipyard.packages.test-safehold-service
            pkgs.mprocs
          ];
        };

        devShells.androidDev = pkgs.mkShell {
          inputsFrom = [
            inputs'.p2p-shipyard.devShells.holochainTauriAndroidDev
            devShells.default
          ];
        };

        devShells.ci = pkgs.mkShell {
          inputsFrom = [
            inputs'.p2p-shipyard.devShells.holochainTauriAndroidDev
            inputs'.p2p-shipyard.devShells.synchronized-pnpm
            inputs'.p2p-shipyard.devShells.default
          ];
        };
      };
    };
}
