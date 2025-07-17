{ inputs, ... }:
let

  sshPubKeys = {
    guillem =
      "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIDTE+RwRfcG3UNTOZwGmQOKd5R+9jN0adH4BIaZvmWjO guillem.cordoba@gmail.com";
  };
  sshModule = {
    users.users.root.openssh.authorizedKeys.keys =
      builtins.attrValues sshPubKeys;
    services.openssh.settings.PermitRootLogin = "without-password";
  };

  bootstrapUrl = "http://157.180.93.55:8888";

  path = "/root/dash-chat/v0.3.x";

  always_online_module = {
    systemd.services.dash_chat_aon1 = let
      aon =
        inputs.p2p-shipyard.inputs.always-online-nodes.outputs.builders."aarch64-linux".aon-for-happs {
          happs =
            [ inputs.self.outputs.packages."x86_64-linux".dash_chat_happ ];
        };
    in {
      enable = true;
      path = [ aon ];
      wantedBy = [ "multi-user.target" ];
      serviceConfig = {
        ExecStart =
          "${aon}/bin/always-online-node --data-dir ${path}/aon --bootstrap-url ${bootstrapUrl}";
        Restart = "always";
        RestartSec = 10;
      };
    };
    systemd.services.dash_chat_aon2 = let
      aon =
        inputs.p2p-shipyard.inputs.always-online-nodes.outputs.builders."aarch64-linux".aon-for-happs {
          happs =
            [ inputs.self.outputs.packages."x86_64-linux".dash_chat_happ ];
        };
    in {
      enable = true;
      path = [ aon ];
      wantedBy = [ "multi-user.target" ];
      serviceConfig = {
        ExecStart = "${aon}/bin/always-online-node --data-dir ${path}/aon2";
        Restart = "always";
        RestartSec = 10;
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
            garnix.server.persistence.name = "dash-chat-aon-v0-3-x-1";
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
            garnix.server.persistence.name = "dash-chat-aon-v0-3-x-2";
            system.stateVersion = "25.05";
            garnix.server.enable = true;
            garnix.server.persistence.enable = true;
          }
        ];
      };
    };
  };
}

