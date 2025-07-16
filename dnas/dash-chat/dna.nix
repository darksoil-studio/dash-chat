{ inputs, ... }:

{
  # Import all ./zomes/coordinator/*/zome.nix and ./zomes/integrity/*/zome.nix  
  imports = (map (m: "${./.}/zomes/coordinator/${m}/zome.nix")
    (builtins.attrNames (builtins.readDir ./zomes/coordinator)));
  # ++ (map (m: "${./.}/zomes/integrity/${m}/zome.nix")
  #   (builtins.attrNames (builtins.readDir ./zomes/integrity)));
  perSystem = { inputs', self', lib, system, ... }: {
    packages.dash_chat_dna =
      inputs.p2p-shipyard.outputs.builders.${system}.dna {
        dnaManifest = ./workdir/dna.yaml;
        zomes = {
          messenger_integrity =
            inputs'.p2p-shipyard.packages.messenger_integrity;
          messenger = inputs'.p2p-shipyard.builders.messenger {
            linked_devices_coordinator_zome_name = "linked_devices";
            async_message_zome_name = "safehold_async_messages";
          };

          linked_devices_integrity =
            inputs'.p2p-shipyard.packages.linked_devices_integrity;
          linked_devices = inputs'.p2p-shipyard.packages.linked_devices;

          friends_integrity = inputs'.p2p-shipyard.packages.friends_integrity;
          friends = inputs'.p2p-shipyard.builders.friends {
            linked_devices_coordinator_zome_name = "linked_devices";
            async_message_zome_name = "safehold_async_messages";
          };

          encrypted_messages = inputs'.p2p-shipyard.packages.encrypted_messages;
          encrypted_messages_integrity =
            inputs'.p2p-shipyard.packages.encrypted_messages_integrity;

          safehold_async_messages = self'.packages.safehold_async_messages;
        };
      };
  };
}
