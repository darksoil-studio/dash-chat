{ inputs, ... }:

{
  perSystem = { inputs', lib, self', system, ... }: {
    packages.aon = (inputs.aons.outputs.builders.${system}.aon-for-dna {
      dna_bundle = self'.packages.messenger_demo_dna;
    });

  };
}
