{ inputs, ... }:

{
  # Import all ./zomes/coordinator/*/zome.nix and ./zomes/integrity/*/zome.nix  
  # imports = (
  #     map (m: "${./.}/zomes/coordinator/${m}/zome.nix")
  #       (builtins.attrNames (builtins.readDir ./zomes/coordinator))
  #   )
  #   ++ 
  #   (
  #     map (m: "${./.}/zomes/integrity/${m}/zome.nix")
  #       (builtins.attrNames (builtins.readDir ./zomes/integrity))
  #   )
  # ;
  perSystem = { inputs', self', lib, system, ... }: {
    packages.dash_chat_dna =
      inputs.holochain-nix-builders.outputs.builders.${system}.dna {
        dnaManifest = ./workdir/dna.yaml;
        zomes = {
          # notifications_integrity =
          #   inputs'.notifications-zome.packages.notifications_integrity;
          # notifications = inputs'.notifications-zome.packages.notifications;
          messenger_integrity =
            inputs'.messenger-zome.packages.messenger_integrity;
          messenger = inputs'.messenger-zome.packages.messenger;
          # Include here the zome packages for this DNA, e.g.:
          linked_devices_integrity =
            inputs'.linked-devices-zome.packages.linked_devices_integrity;
          linked_devices = inputs'.linked-devices-zome.packages.linked_devices;
          friends_integrity = inputs'.friends-zome.packages.friends_integrity;
          friends = inputs'.friends-zome.packages.friends;
        };
      };
  };
}
