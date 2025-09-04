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

  bootstrapUrl =
    "https://bootstrap.kitsune-v0-2.kitsune.darksoil-studio.garnix.me";

  always_online_module = {
    systemd.services.dash_chat_aon = let
      aon =
        inputs.p2p-shipyard.inputs.always-online-nodes.outputs.builders."x86_64-linux".aon-for-happs {
          happs = [ inputs.self.outputs.packages."x86_64-linux".dash_chat_happ ];
        };
    in {
      enable = true;
      path = [ aon ];
      wantedBy = [ "multi-user.target" ];
      after = [ "network-online.target" ];
      wants = [ "network-online.target" ];
      serviceConfig = {
        ExecStart =
          "${aon}/bin/always-online-node --data-dir /root/aon --bootstrap-url ${bootstrapUrl}";
        RuntimeMaxSec = "3600"; # Restart every hour
        Restart = "always";
      };
    };
  };

in {
  flake = {
    nixosConfigurations = {
      aon1 = inputs.nixpkgs.lib.nixosSystem {
        system = "x86_64-linux";
        modules = [
          inputs.garnix-lib.nixosModules.garnix
          sshModule
          always_online_module
          {
            garnix.server.persistence.name = "dash-chat-aon-v0-7-0-1";
            system.stateVersion = "25.05";
            garnix.server.enable = true;
            garnix.server.persistence.enable = true;
          }
        ];
      };
      aon2 = inputs.nixpkgs.lib.nixosSystem {
        system = "x86_64-linux";
        modules = [
          inputs.garnix-lib.nixosModules.garnix
          sshModule
          always_online_module
          {
            garnix.server.persistence.name = "dash-chat-aon-v0-7-0-2";
            system.stateVersion = "25.05";
            garnix.server.enable = true;
            garnix.server.persistence.enable = true;
          }
        ];
      };
    };
  };
}

