{ inputs, ... }:

{
  perSystem = { inputs', system, ... }: {
    packages.safehold_async_messages =
      inputs.holochain-nix-builders.outputs.builders.${system}.rustZome {
        workspacePath = inputs.self.outPath;
        crateCargoToml = ./Cargo.toml;
      };
  };
}

