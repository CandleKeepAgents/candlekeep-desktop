# CandleKeep Desktop

macOS menu bar app for managing CandleKeep CLI, plugins, and integrations. Built with Tauri v2 + React/TypeScript.

## Tech Stack

- **Backend**: Rust (Tauri v2) — tray icon, shell commands, API client
- **Frontend**: React 19 + TypeScript + Tailwind CSS 4 + Vite
- **Distribution**: DMG via GitHub Releases (unsigned, alpha)

## Project Structure

```
src-tauri/src/
├── main.rs              # Binary entry point
├── lib.rs               # Tauri setup: tray icon, menu, commands, no-dock-icon
├── commands/
│   ├── system.rs        # Homebrew/cargo/node/Xcode detection + get_full_path() helper
│   ├── cli.rs           # ck CLI detection, install, update, auth status
│   ├── plugin.rs        # Claude Code plugin detection, install, update
│   └── metrics.rs       # CandleKeep API client (whoami, metrics)
├── integrations/
│   ├── mod.rs           # Integration trait + IntegrationInfo/IntegrationStatus types
│   ├── claude_code.rs   # Full implementation
│   ├── cursor.rs        # Stub (Coming Soon)
│   └── codex.rs         # Stub (Coming Soon)
├── state.rs             # App state persistence (~/.candlekeep/desktop-state.json)
└── updater.rs           # Self-update via GitHub Releases API

src/
├── App.tsx              # Root: setup check → wizard or dashboard + nav
├── App.css              # Tailwind import + scrollbar hiding + zinc-900 bg
├── main.tsx             # React mount + macOS touch event workaround
├── pages/
│   ├── Dashboard.tsx    # Status cards, metrics, update banners, quick actions
│   ├── Setup.tsx        # Multi-step first-run wizard
│   ├── Integrations.tsx # Claude Code + future IDE integrations
│   ├── ReleaseNotes.tsx # GitHub Releases API
│   └── Settings.tsx     # About, updates, sign out
├── components/          # StatusCard, MetricsCard, UpdateBanner, QuickActions, IntegrationCard
├── hooks/               # useCliStatus, usePluginStatus, useMetrics (all polling)
└── lib/
    ├── tauri-commands.ts # Type-safe invoke() wrappers for all Rust commands
    └── types.ts          # Shared TS types matching Rust serde structs
```

## Key Patterns

### macOS GUI PATH Issue
macOS GUI apps do NOT inherit the user's shell PATH. All binary detection must:
1. Check known filesystem paths directly (`/opt/homebrew/bin/`, `~/.local/bin/`, `~/.cargo/bin/`)
2. Use `get_full_path()` from `commands/system.rs` when spawning any shell command

### Plugin Detection Path
The CandleKeep plugin lives at:
```
~/.claude/plugins/marketplaces/candlekeep/plugins/candlekeep-cloud/
```
NOT `~/.claude/plugins/candlekeep-marketplace/` (that path doesn't exist).

### Tauri IPC Contract
Every `#[tauri::command]` in Rust must have a matching wrapper in `src/lib/tauri-commands.ts`.
Return types use `Result<T, String>` in Rust → the TS wrapper gets `T` on success, throws on error.
Struct fields use `snake_case` in Rust and are received as `snake_case` in TypeScript (Tauri does NOT convert to camelCase).

### Window Behavior
- **No dock icon**: `app.set_activation_policy(tauri::ActivationPolicy::Accessory)` in `lib.rs`
- **Transparent + decorations off**: configured in `tauri.conf.json`
- **Click-through fix**: touch event blocking in `main.tsx` (macOS WebKit bug with transparent windows)
- **Draggable header**: `onMouseDown → getCurrentWindow().startDragging()` in `App.tsx`
- **Close = hide**: X button calls `getCurrentWindow().hide()`, tray click toggles visibility

### CandleKeep API
- Base URL: `https://www.getcandlekeep.com/api/v1`
- Auth: `Authorization: Bearer {api_key}` from `~/.candlekeep/config.toml`
- Key endpoint: `GET /auth/whoami` → `{ id, email, name, tier, item_limit, item_count }`

### CLI Auth Flow
`ck auth login` starts a local TCP server, opens the browser to `/cli-auth?port=N`, captures the API key from the callback, and saves to `~/.candlekeep/config.toml`.

## Commands

```bash
pnpm tauri dev          # Run in dev mode (tray icon + hot reload)
pnpm tauri build        # Build DMG for distribution
cargo check --manifest-path src-tauri/Cargo.toml  # Rust type check
pnpm exec tsc --noEmit  # TypeScript type check
```

## Adding a New Tauri Command

1. Add the function in the appropriate `src-tauri/src/commands/*.rs` file with `#[tauri::command]`
2. Register it in `lib.rs` → `invoke_handler(tauri::generate_handler![...])`
3. Add a typed wrapper in `src/lib/tauri-commands.ts`
4. Add any new types to `src/lib/types.ts`

## Adding a New Integration

1. Create `src-tauri/src/integrations/new_ide.rs` implementing the `Integration` trait
2. Add `pub mod new_ide;` to `integrations/mod.rs`
3. Add an `IntegrationCard` entry in `src/pages/Integrations.tsx`
