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

  path = "/root/dash-chat/v0.3.x";

  push_notifications_service_provider =
    inputs.push-notifications-service.outputs.packages."x86_64-linux".push-notifications-service-provider;

  push_notifications_service_provider_module = {
    systemd.services.push_notifications_service_provider1 = {
      enable = true;
      path = [ push_notifications_service_provider ];
      wantedBy = [ "multi-user.target" ];
      serviceConfig = {
        ExecStart =
          "${push_notifications_service_provider}/bin/push-notifications-service-provider --data-dir ${path}/push-notifications-service-provider1";
        RuntimeMaxSec = "3600"; # Restart every hour

        Restart = "always";
        RestartSec = 1;
      };
    };
    systemd.services.push_notifications_service_provider2 = {
      enable = true;
      path = [ push_notifications_service_provider ];
      wantedBy = [ "multi-user.target" ];
      serviceConfig = {
        ExecStart =
          "${push_notifications_service_provider}/bin/push-notifications-service-provider --data-dir ${path}/push-notifications-service-provider2";
        RuntimeMaxSec = "3600"; # Restart every hour

        Restart = "always";
        RestartSec = 1;
      };
    };
  };

  safehold_service_provider =
    inputs.safehold.outputs.packages."x86_64-linux".safehold-service-provider;

  safehold_service_provider_module = {
    systemd.services.safehold_service_provider1 = {
      enable = true;
      path = [ safehold_service_provider ];
      wantedBy = [ "multi-user.target" ];
      serviceConfig = {
        ExecStart =
          "${safehold_service_provider}/bin/safehold-service-provider --data-dir ${path}/safehold-service-provider1";
        RuntimeMaxSec = "3600"; # Restart every hour

        Restart = "always";
        RestartSec = 1;
      };
    };
    systemd.services.safehold_service_provider2 = {
      enable = true;
      path = [ safehold_service_provider ];
      wantedBy = [ "multi-user.target" ];
      serviceConfig = {
        ExecStart =
          "${safehold_service_provider}/bin/safehold-service-provider --data-dir ${path}/safehold-service-provider2";
        RuntimeMaxSec = "3600"; # Restart every hour

        Restart = "always";
        RestartSec = 1;
      };
    };
  };

  always_online_module = {
    systemd.services.dash_chat_aon1 = let
      aon = inputs.aons.outputs.builders."aarch64-linux".aon-for-happs {
        happs = [ inputs.self.outputs.packages."x86_64-linux".dash_chat_happ ];
      };
    in {
      enable = true;
      path = [ aon ];
      wantedBy = [ "multi-user.target" ];
      serviceConfig = {
        ExecStart = "${aon}/bin/always-online-node --data-dir ${path}/aon1";
        Restart = "always";
        RestartSec = 10;
      };
    };
    systemd.services.dash_chat_aon2 = let
      aon = inputs.aons.outputs.builders."aarch64-linux".aon-for-happs {
        happs = [ inputs.self.outputs.packages."x86_64-linux".dash_chat_happ ];
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
      server = inputs.nixpkgs.lib.nixosSystem {
        system = "x86_64-linux";
        modules = [
          inputs.garnix-lib.nixosModules.garnix
          sshModule
          push_notifications_service_provider_module
          safehold_service_provider_module
          always_online_module
          {
            garnix.server.persistence.name = "server1";
            system.stateVersion = "25.05";
            garnix.server.enable = true;
            garnix.server.persistence.enable = true;
          }
        ];
      };
    };
  };
}

