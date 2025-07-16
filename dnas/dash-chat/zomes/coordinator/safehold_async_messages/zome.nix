{ inputs, ... }:

{
  perSystem = { inputs', system, ... }: {
    packages.safehold_async_messages =
      inputs.p2p-shipyard.outputs.builders.${system}.rustZome {
        workspacePath = inputs.self.outPath;
        crateCargoToml = ./Cargo.toml;
      };
  };
}

