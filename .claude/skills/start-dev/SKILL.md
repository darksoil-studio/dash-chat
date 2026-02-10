---
name: start-dev
description: Start the Dash Chat development environment (Tauri agents, UI dev server, mailbox server, stores watcher). Use this when you need to run and test the app.
user-invocable: false
---

# Start Development Environment

Start all the processes needed to run Dash Chat locally. Do NOT use `pnpm start` or `mprocs` — they require an interactive TTY.

Use the Bash tool with `run_in_background: true` for each process so you get task IDs you can later stop with `TaskStop`.

## Step 1: Allocate free ports and temp directory

Run these commands to allocate ports and a temp DB directory that won't conflict with any running instance:

```bash
UI_PORT=$(node -e "const s=require('net').createServer();s.listen(0,()=>{console.log(s.address().port);s.close()})")
MAILBOX_PORT=$(node -e "const s=require('net').createServer();s.listen(0,()=>{console.log(s.address().port);s.close()})")
DEV_DBS_PATH=$(mktemp -d)
```

## Step 2: Build stores

```bash
pnpm -F ./packages/stores build
```

## Step 3: Start all processes in the background

Launch each of these as a **separate background Bash task** (using `run_in_background: true`). Save the task IDs so you can stop them later.

1. **Stores watcher** (rebuilds on changes):
   ```bash
   pnpm -F ./packages/stores dev
   ```

2. **Mailbox server**:
   ```bash
   mkdir -p "$DEV_DBS_PATH/mailbox-server"
   cargo run -p mailbox-server -- --db-path "$DEV_DBS_PATH/mailbox-server/mailbox.db" --addr "0.0.0.0:$MAILBOX_PORT"
   ```

3. **UI dev server** (Vite):
   ```bash
   UI_PORT=$UI_PORT pnpm -F ./ui start
   ```

4. **Tauri agent 1** (Rust backend — connects to the UI dev server):
   ```bash
   AGENT=1 MAILBOX_PORT=$MAILBOX_PORT DEV_DBS_PATH=$DEV_DBS_PATH pnpm tauri dev --config "{\"build\":{\"devUrl\":\"http://localhost:$UI_PORT\"}}"
   ```

5. **Tauri agent 2** (optional — only start if testing p2p communication):
   ```bash
   AGENT=2 MAILBOX_PORT=$MAILBOX_PORT DEV_DBS_PATH=$DEV_DBS_PATH pnpm tauri dev --config "{\"build\":{\"devUrl\":\"http://localhost:$UI_PORT\"}}"
   ```

## Step 4: Wait for the app to launch and discover MCP bridge ports

Poll each agent's task output for `MCP Bridge plugin initialized` — that means the app is ready.

**IMPORTANT: The MCP bridge ports are dynamically assigned.** They default to 9223/9224 but will increment if those ports are already in use (e.g., 9225, 9226, etc.). You **must** extract the actual port from the log line:

```
[MCP][PLUGIN][INFO] MCP Bridge plugin initialized for 'Dash Chat' (studio.darksoil.dashchat) on 0.0.0.0:PORT
```

Use `grep` on the task output files to find the actual ports:
```bash
grep "MCP Bridge plugin initialized" /path/to/agent1-task-output
grep "MCP Bridge plugin initialized" /path/to/agent2-task-output
```

## Step 5: Connect via Tauri MCP bridge

Use `driver_session` (start) with the **actual port** from Step 4 (not hardcoded 9223) to connect to the running Tauri instance, then use `webview_screenshot`, `webview_dom_snapshot`, and other Tauri MCP tools to inspect and interact with the UI.

When running **two agents**, each gets its own MCP bridge on a different port. Use the `appIdentifier` parameter (set to the port number) to target a specific agent:

```
driver_session(action: start, port: <agent1-port>)  # connects to Agent 1
driver_session(action: start, port: <agent2-port>)  # connects to Agent 2

webview_screenshot(appIdentifier: <agent1-port>)     # screenshot Agent 1
webview_screenshot(appIdentifier: <agent2-port>)     # screenshot Agent 2
```

Both sessions can be active simultaneously. The most recently connected app becomes the default (used when `appIdentifier` is omitted).

## Cleanup: Stop all dev processes

When done testing, stop the driver session and use `TaskStop` on each of the background task IDs you saved in Step 3.
