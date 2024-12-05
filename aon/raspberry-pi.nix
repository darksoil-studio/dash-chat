{ inputs, self, ... }:

{
  perSystem = { inputs', self', lib, system, ... }: {
    rpi-aon-iso = inputs.nixos-generators.nixosGenerate {
      system = "aarch64-linux";
      modules = [
        ({ inputs, config, pkgs, ... }:
          let
            user = "rpi";
            password = "hablaamigo";
          in {

            # imports = [ <nixpkgs/nixos/modules/installer/sd-card/sd-image-aarch64.nix> ];

            # boot = {
            #   kernelPackages = pkgs.linuxKernel.packages.linux_rpi4;
            #   initrd.availableKernelModules = [ "xhci_pci" "usbhid" "usb_storage" ];
            #   loader = {
            #     grub.enable = false;
            #     generic-extlinux-compatible.enable = true;
            #   };
            # };

            # fileSystems = {
            #   "/" = {
            #     device = "/dev/disk/by-label/NIXOS_SD";
            #     fsType = "ext4";
            #     options = [ "noatime" ];
            #   };
            # };

            networking = { hostName = "rpimaster"; };

            networking.wireless = {
              enable = true;
              interfaces = [ "wlan0" ];
              networks = let
                SSID = builtins.getEnv ("SSID");
                SSID-PASSWORD = builtins.getEnv ("SSIDPASSWORD");
              in { ${SSID} = { psk = SSID-PASSWORD; }; };
            };

            users = {
              mutableUsers = false;
              users."${user}" = {
                isNormalUser = true;
                password = password;
                extraGroups = [ "wheel" ];
              };
            };

            environment.systemPackages = with pkgs; [ vim ];

            services.create_ap = {
              enable = true;
              settings = {
                INTERNET_IFACE = "eth0";
                WIFI_IFACE = "wlan0";
                SSID = "rpi";
                PASSPHRASE = "12345678";
              };
            };

            # users.extraUsers.nixos.openssh.authorizedKeys.keys = [ "ssh-ed25519 ...." ];

            # This is required so that pod can reach the API server (running on port 6443 by default)
            # networking.firewall.allowedTCPPorts = [ 6443 80 ];

            services = {
              openssh = {
                enable = true;
                settings.PasswordAuthentication = false;
              };
            };

            systemd.services.messenger_aon = {
              enable = true;
              path = [
                inputs.aons.outputs.packages."aarch64-linux".always-online-node
              ];
              serviceConfig = {
                Restart = "always";
                ExecStart = let
                  homeDir = config.users.users.${user}.home;
                  dna = self.outputs.packages."x86_64-linux".messenger_demo_dna;
                in "mkdir -p ${homeDir}/messenger_aon && always-only-node ${dna} --data-dir ${homeDir}/messenger_aon";
              };
            };

            system.stateVersion = "24.05";
          })
      ];
      format = "iso";

      specialArgs = { inherit inputs; };

      # formatConfigs = {
      #   sd-aarch64 = { lib, ... }: { sdImage.compressImage = false; };
      # };

    };
  };
}
