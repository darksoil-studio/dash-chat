{ inputs, self, ... }:

{
  perSystem = { inputs', self', lib, system, ... }: {
    packages.rpi-aon = inputs.nixos-generators.nixosGenerate {
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
            # Disabling the whole `profiles/base.nix` module, which is responsible
            # for adding ZFS and a bunch of other unnecessary programs:
            disabledModules = [ "profiles/base.nix" ];

            # fileSystems = {
            #   "/" = {
            #     device = "/dev/disk/by-label/NIXOS_SD";
            #     fsType = "ext4";
            #     options = [ "noatime" ];
            #   };
            # };

            networking = {
              hostName = "rpimaster";
              firewall.enable = false;
              wireless = {
                enable = true;
                interfaces = [ "wlan0" ];
                # networks = let
                #   SSID = "Pixel_4841";
                #   SSID-PASSWORD = "12344321";
                # in { ${SSID} = { psk = SSID-PASSWORD; }; };
              };
            };

            services.avahi = {
              nssmdns4 = true;
              enable = true;
              ipv4 = true;
              ipv6 = true;
              publish = {
                enable = true;
                addresses = true;
                workstation = true;
              };
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
                INTERNET_IFACE = "end0";
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
                # settings.PasswordAuthentication = false;
              };
            };

            systemd.services.dash_chat_aon = let
              homeDir = config.users.users.${user}.home;
              aon =
                inputs.p2p-shipyard.inputs.always-online-nodes.outputs.builders."aarch64-linux".aon-for-happs {
                  happs =
                    [ self.outputs.packages."x86_64-linux".dash_chat_happ ];
                };
            in {
              enable = true;
              path = [ aon ];
              wantedBy = [ "multi-user.target" ];
              serviceConfig = {
                ExecStart =
                  "${aon}/bin/always-online-node --data-dir ${homeDir}";
                Restart = "always";
                RestartSec = 10;
              };
            };

            system.stateVersion = "24.05";
          })
      ];
      format = "sd-aarch64";

      specialArgs = { inherit inputs; };

    };
  };
}
