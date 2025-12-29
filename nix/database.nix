{ inputs, ... }:
let

  sshPubKeys = {
    guillem =
      "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIDTE+RwRfcG3UNTOZwGmQOKd5R+9jN0adH4BIaZvmWjO guillem.cordoba@gmail.com";
    guillemslaptop =
      "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIO8DVpvRgQ90MyMyiuNdvyMNAio9n2o/+57MyhZS2A5A guillem.cordoba@gmail.com";
  };

  sshModule = {
    users.users.root.openssh.authorizedKeys.keys =
      builtins.attrValues sshPubKeys;
    services.openssh.enable = true;
    services.openssh.settings.PermitRootLogin = "without-password";
  };

  # bootstrapUrl =
  #   "https://bootstrap.kitsune-v0-1.kitsune.darksoil-studio.garnix.me";

  cassandra_module = {
    services.cassandra = {
      enable = true;
    };
  };

in {
  flake = {
    nixosConfigurations = {
      db1 = inputs.nixpkgs.lib.nixosSystem {
        system = "x86_64-linux";
        modules = [
          inputs.garnix-lib.nixosModules.garnix
          sshModule
          cassandra_module
          {
            garnix.server.persistence.name = "db-v0-6-0-1";
            system.stateVersion = "25.05";
            garnix.server.enable = true;
            garnix.server.persistence.enable = true;
          }
        ];
      };
    };
  };
}

