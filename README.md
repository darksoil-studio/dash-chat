# Dash Chat

## Environment Setup

> PREREQUISITE: set up the [holochain development environment](https://developer.holochain.org/docs/install/).

Enter the nix shell by running this in the root folder of the repository: 

```bash
nix develop
pnpm install
```

**Run all the other instructions in this README from inside this nix shell, otherwise they won't work**.

## Running 2 agents
 
```bash
pnpm start
```

This will create a network of 2 nodes connected to each other and their respective UIs.
It will also bring up the Holochain Playground for advanced introspection of the conductors.

## Running the backend tests

```bash
pnpm test
```

## Bootstrapping a network

Create a custom network of nodes connected to each other and their respective UIs with:

```bash
pnpm network 3
```

Substitute the "3" for the number of nodes that you want to bootstrap in your network.
This will also bring up the Holochain Playground for advanced introspection of the conductors.

## Packaging

To package the web happ:
``` bash
pnpm package
```

You'll have the `dash-chat.webhapp` in `workdir`. This is what you should distribute so that the Holochain Launcher can install it.

## Documentation

This repository is using these tools:
- [PNPM Workspaces](https://pnpm.io/workspaces): npm replacement that supports git dependencies with subdirectories.
- [hc](https://github.com/holochain/holochain/tree/develop/crates/hc): Holochain CLI to easily manage Holochain development instances.
- [@holochain/tryorama](https://www.npmjs.com/package/@holochain/tryorama): test framework.
- [@holochain/client](https://www.npmjs.com/package/@holochain/client): client library to connect to Holochain from the UI.
- [@holochain-playground/cli](https://www.npmjs.com/package/@holochain-playground/cli): introspection tooling to understand what's going on in the Holochain nodes.
