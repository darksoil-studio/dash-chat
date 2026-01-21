# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Dash Chat is an end-to-end encrypted messenger built with Svelte 5 (frontend) and Rust/Tauri (backend), using p2panda for peer-to-peer communication. The application works both with and without internet connectivity.

**Current Status**: Pre-alpha, being rebuilt on top of p2panda.

## General Coding Style

Please read this coding style carefully and take it into account when planning or coding:

- Try to remain as simple as possible with your implementations.
- Try to reuse types and functions across the project rather than reimplement them.
- Don't use `any` or `unknown` typescript types. Rather, try to understand the actual typescript types and use them to infer the appropriate data structures and algorithms to use.

## Development Environment

### Prerequisites
- Rust (https://rust-lang.org/tools/install/)
- pnpm (version >=9.0.0)
- Tauri prerequisites for your platform (https://tauri.app/start/prerequisites/)
- Alternatively: Use `nix develop` for a Nix development shell

### Initial Setup
```bash
pnpm install
```

## Common Commands

### Running the Application
```bash
# Start two instances forming a p2panda network
pnpm start

# This uses mprocs to spawn multiple processes:
# - agent1 and agent2: Two Tauri development instances
# - ui: Frontend development server
# - stores: Watches and rebuilds the stores package
```

### Development Tasks
```bash
# Run Rust tests
cargo test
# or
pnpm test

# Type check Svelte components (from ui/ directory)
pnpm check
pnpm check:watch

# Build UI (from ui/ directory)
pnpm build

# Build stores package (from packages/stores/ directory)
pnpm build
```

### Mobile Development
```bash
# Run on Android
pnpm tauri android dev

# View Android logs
adb logcat | grep -F "`adb shell ps | grep studio.darksoil.dashchat | tr -s [:space:] ' ' | cut -d' ' -f2`"
```

## Architecture

### Monorepo Structure

This is a pnpm workspace with multiple packages:
- **ui/**: Svelte 5 + TypeScript frontend (SvelteKit application)
- **packages/stores/**: Shared TypeScript stores for state management
- **crates/dashchat-node/**: Core p2p backend logic (Rust)
- **crates/mailbox-server/**: HTTP server for offline message storage
- **src-tauri/**: Tauri application wrapper and integration layer
- **site/**: Marketing/download site

### Backend Architecture (Rust)

**Main Components:**

1. **dashchat-node** (`crates/dashchat-node/`):
   - Core p2p networking logic built on p2panda
   - Key modules:
     - `node.rs`: Main Node implementation with p2panda integration
     - `chat.rs` & `contact.rs`: Chat and contact management
     - `spaces.rs` & `topic.rs`: Space and topic abstractions
     - `stores/`: Data persistence layer
     - `polestar/`: Additional p2panda functionality
   - Uses p2panda libraries from custom fork: `https://github.com/maackle/p2panda.git` (branch: dashchat)

2. **src-tauri** (Tauri app layer):
   - `lib.rs`: Application setup, plugin initialization, and node lifecycle
   - `commands/`: Tauri command handlers that bridge frontend to backend:
     - `logs.rs`: Operation log queries
     - `profile.rs`: User profile management
     - `contacts.rs`: Contact management
     - `devices.rs`: Device management
     - `chats.rs` & `group_chat.rs`: Chat functionality
   - `push_notifications.rs`: Mobile push notification handling
   - `menu.rs`: Desktop menu configuration
   - `utils.rs`: Shared utilities

3. **mailbox-server** (`crates/mailbox-server/`):
   - Standalone HTTP server for storing/retrieving encrypted message blobs
   - Built with Axum web framework and redb embedded database
   - Key modules:
     - `lib.rs`: App initialization, routing, and database setup
     - `store_blobs.rs`: POST `/blobs/store` endpoint for storing blobs
     - `get_blobs.rs`: POST `/blobs/get` endpoint for retrieving blobs with sync support
     - `cleanup.rs`: Background task that deletes messages older than 7 days
     - `blob.rs`: Base64-encoded binary data wrapper
   - Data model:
     - Key format: `topic_id:log_id:sequence_number:uuid_v7`
     - Blobs organized by topic → log → sequence number hierarchy
     - UUID v7 suffix enables time-based cleanup
   - Features bidirectional sync: returns missing blobs to client AND requests blobs the server is missing
   - Run with: `cargo run --bin mailbox-server -- --db-path <path> --addr <addr>`

**Key Backend Patterns:**
- Node managed as Tauri state (accessed via `app.state::<Node>()`)
- Async notification channel from Node to frontend via Tauri events
- All backend commands are async and return `Result<T, String>`
- Uses p2panda's operation-based data model with CBOR encoding

### Frontend Architecture (Svelte 5 + TypeScript)

**Structure:**
- **ui/src/routes/**: SvelteKit file-based routing
  - Main routes: contacts, direct-messages, group-chat, settings, add-contact, new-group, new-message
  - Uses Svelte 5 runes (signals) for reactivity
- **ui/src/components/**: Reusable UI components
- **ui/src/utils/**: Utility functions (image compression, time formatting, QR codes, etc.)
- **packages/stores/src/**: Shared state management
  - Organized by domain: contacts, chats, group-chats, direct-messages, devices
  - Each domain has a `-store.ts` (state) and `-client.ts` (Tauri commands)
  - `p2panda/`: Core p2panda integration (logs-store, logs-client, types)

**Frontend Patterns:**
- Signalium for reactive state management
- Tauri commands invoked via `invoke()` from `@tauri-apps/api`
- UI built with Konsta UI components (mobile-first design)
- Internationalization using @inlang/paraglide-js
- Image compression before upload

### Data Flow

1. User action in Svelte UI
2. Svelte store calls client function
3. Client invokes Tauri command (crosses JS/Rust boundary)
4. Command handler in src-tauri/commands/ processes request
5. Interacts with Node (dashchat-node crate)
6. Node performs p2panda operations (log operations, sync, discovery)
7. Results returned through Tauri command response
8. Async updates pushed via Tauri events to frontend
9. Frontend stores react to updates and UI re-renders

### P2Panda Integration

The app uses p2panda for:
- Distributed log-based data structures
- End-to-end encryption
- Peer discovery (mDNS)
- Data synchronization between nodes
- Spaces for grouping related data

Core p2panda dependencies (from custom fork):
- p2panda-core: Core types and operations
- p2panda-auth: Authentication
- p2panda-encryption: E2EE
- p2panda-net: Networking layer
- p2panda-sync: Synchronization logic
- p2panda-spaces: Space management
- p2panda-discovery: Peer discovery (mDNS)

## CI

Execute all CI commands inside of the default nix shell with `nix develop`.

## Testing

### Rust Tests
```bash
cargo test
```

Run tests from workspace root. Tests use tokio async runtime.

### Development Testing
Use `pnpm start` to run two instances locally that can communicate with each other over the p2panda network.

## Platform Support

- **Desktop**: Linux, macOS, Windows (via Tauri)
- **Mobile**: Android and iOS support
  - Android-specific: barcode scanner, push notifications
  - iOS-specific: barcode scanner, push notifications, safe area insets

## Important Notes

- **P2panda fork**: This project uses a custom fork of p2panda. Do not update p2panda dependencies without checking compatibility.
- **Rust edition**: Uses Rust edition 2021 (src-tauri) and 2024 (dashchat-node)
- **Nightly features**: dashchat-node uses `#![feature(bool_to_result)]`
- **Mobile vs Desktop**: Code paths differ for mobile/desktop (check `#[cfg(mobile)]` and `#[cfg(not(mobile))]`)
- **Internationalization**: UI supports multiple languages via Weblate integration

## Build Configuration

### Development
Standard development builds with debug symbols.

### Release
Optimized builds with:
- opt-level 3
- LTO enabled ("fat")
- Single codegen unit
- Panic = abort

## Localization

Translations managed through Weblate: https://hosted.weblate.org/projects/dash-chat
Contact team at hello@dashchat.org to become a translation reviewer.

