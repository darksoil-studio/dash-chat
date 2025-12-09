# Dash Chat

Dash Chat is an end to end encrypted messenger that works with internet, without internet and bridges between the two. As long as there is a way for devices to communicate with each other, Dash Chat works.

## 🚧 Dash Chat is in Pre-alpha 🚧

Dash Chat is in pre-alpha. We are currently rebuilding the application on top of [p2panda](https://github.com/p2panda/p2panda).

## Tech Stack

- Frontend: Svelte 5 with TypeScript
- Backend: Rust with Tauri
- P2P: P2Panda for peer-to-peer communication
- Build Tool: Vite

## Developer setup

1. Install [Rust](https://rust-lang.org/tools/install/).
2. Install [pnpm](https://pnpm.io/).
3. Install the [Tauri pre-requisits](https://tauri.app/start/prerequisites/) for your platform.
4. Run `pnpm install`.

  OR

If you use nix, just use `nix develop` to enter the development shell and run `pnpm install` to install the `pnpm` dependencies.


### Running the app

To run the app, run this command:

```bash
pnpm start
```

This will spawn two instances of the tauri, forming a p2panda network of 2 nodes.
