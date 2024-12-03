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
    packages.messenger_demo_dna =
      inputs.tnesh-stack.outputs.builders.${system}.dna {
        dnaManifest = ./workdir/dna.yaml;
        zomes = {
          file_storage_integrity = inputs'.file-storage.packages.file_storage_integrity;
          file_storage = inputs'.file-storage.packages.file_storage;
          # Include here the zome packages for this DNA, e.g.:
          # profiles_integrity = inputs'.profiles-zome.packages.profiles_integrity;
          # This overrides all the "bundled" properties for the DNA manifest
          linked_devices_integrity =
            inputs'.linked-devices-zome.packages.linked_devices_integrity;
          linked_devices = inputs'.linked-devices-zome.packages.linked_devices;
          profiles_integrity =
            inputs'.profiles-zome.packages.profiles_integrity;
          profiles = inputs'.profiles-zome.packages.profiles;
        };
      };
  };
}
