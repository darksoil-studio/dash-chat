{ inputs, ... }:

{
  # Import all `dnas/*/dna.nix` files
  imports = (map (m: "${./.}/dnas/${m}/dna.nix") (builtins.attrNames
    (if builtins.pathExists ./dnas then builtins.readDir ./dnas else { })));

  perSystem = { inputs', lib, self', system, ... }: {
    packages.dash_chat_happ =
      inputs.holochain-nix-builders.outputs.builders.${system}.happ {
        happManifest = ./workdir/happ.yaml;
        dnas = {
          # Include here the DNA packages for this hApp, e.g.:
          # my_dna = inputs'.some_input.packages.my_dna;
          # This overrides all the "bundled" properties for the hApp manifest
          main = self'.packages.dash_chat_dna;
          services = inputs'.service-providers.packages.services_dna;
        };
      };
  };
}
