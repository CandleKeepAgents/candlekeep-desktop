# Cross-Platform Desktop + Multi-Host Integration

## Context

CandleKeep Desktop is macOS-only and Claude-only. We need to:
1. Ship on Windows and Linux with full parity
2. Refactor the backend to support multiple hosts (Claude, Cursor, Codex, Amp)
3. Replace ad-hoc macOS code with a structured platform layer

The app is Tauri v2 + React — inherently cross-platform. The work is refactoring the Rust backend and build pipeline.

---

## Architecture: Two New Layers

### Layer 1: Platform Module (`src-tauri/src/platform/`)

Centralizes ALL OS knowledge instead of scattering `#[cfg]` through the codebase.

```
src-tauri/src/platform/
├── mod.rs          # Platform enum, PlatformInfo, get_platform()
├── paths.rs        # OS-specific PATH expansion, binary finder, well-known dirs
├── installer.rs    # Download + extract binaries from GitHub Releases
└── tray.rs         # Tray behavior + Linux fallback
```

**Key types:**
```rust
enum Platform { MacOS, Windows, Linux }

struct PlatformInfo {
    platform: Platform,
    arch: String,           // "aarch64", "x86_64"
    tray_supported: bool,
    paths: PlatformPaths,
}

struct PlatformPaths {
    cli_install_dir: PathBuf,    // ~/.local/bin, %LOCALAPPDATA%\Programs\candlekeep\
    config_dir: PathBuf,         // ~/.candlekeep (all platforms)
    extra_bin_dirs: Vec<PathBuf>, // Platform-specific search paths
    path_separator: char,        // ':' or ';'
}
```

### Layer 2: Integration Manager (`src-tauri/src/integrations/`)

Replace Claude-only `commands/plugin.rs` with adapter pattern:

```
src-tauri/src/integrations/
├── mod.rs              # Integration trait, HostKind enum, manager
├── claude_code.rs      # Existing logic (plugin marketplace flow)
├── cursor.rs           # MCP config install (not Marketplace plugin)
├── codex.rs            # MCP registration
└── amp.rs              # MCP registration
```

**Key types:**
```rust
enum HostKind { ClaudeCode, Cursor, Codex, Amp }

struct IntegrationStatus {
    host: HostKind,
    host_installed: bool,
    integration_installed: bool,
    version: Option<String>,
    latest_version: Option<String>,
    install_method: String,
    status: RequirementStatus,
}

enum RequirementStatus { Satisfied, Missing, Unsupported }
struct ActionResult { ok: bool, message: String, details: Option<String>, restart_required: bool }
```

**Trait each adapter implements:**
```rust
trait HostIntegration {
    fn detect_host(&self, platform: &PlatformInfo) -> bool;
    fn detect_integration(&self) -> IntegrationStatus;
    fn install(&self) -> ActionResult;
    fn update(&self) -> ActionResult;
    fn repair(&self) -> ActionResult;
    fn requirements(&self, platform: &PlatformInfo) -> Vec<Requirement>;
}
```

---

## Generic Tauri Command API

Replace Claude-specific commands with:

```rust
#[tauri::command] fn get_platform_info() -> PlatformInfo;
#[tauri::command] fn check_cli_status() -> CliStatus;
#[tauri::command] fn install_cli() -> ActionResult;
#[tauri::command] fn update_cli() -> ActionResult;
#[tauri::command] fn list_integrations() -> Vec<IntegrationStatus>;
#[tauri::command] fn check_integration(host: HostKind) -> IntegrationStatus;
#[tauri::command] fn install_integration(host: HostKind) -> ActionResult;
#[tauri::command] fn update_integration(host: HostKind) -> ActionResult;
#[tauri::command] fn repair_integration(host: HostKind) -> ActionResult;
```

Keep existing Claude wrappers as TS compatibility shims until React UI is migrated, then delete.

---

## Binary Detection (platform/paths.rs)

### `get_full_path()` — Platform PATH Construction

Currently in `commands/system.rs:7-18`. Move to `platform/paths.rs`:

```rust
pub fn get_full_path() -> String {
    let home = dirs::home_dir().unwrap_or_default();
    let mut paths: Vec<String> = Vec::new();

    // Cross-platform
    paths.push(home.join(".cargo/bin").display().to_string());
    paths.push(home.join(".local/bin").display().to_string());

    #[cfg(target_os = "macos")] {
        paths.extend(["/opt/homebrew/bin", "/opt/homebrew/sbin", "/usr/local/bin"].map(String::from));
    }
    #[cfg(target_os = "linux")] {
        paths.extend(["/usr/local/bin", "/usr/bin", "/snap/bin"].map(String::from));
        paths.push(home.join(".volta/bin").display().to_string());
        // Scan ~/.nvm/versions/node/*/bin for latest
        if let Some(nvm_bin) = find_latest_nvm_bin(&home) { paths.push(nvm_bin); }
    }
    #[cfg(target_os = "windows")] {
        if let Ok(appdata) = std::env::var("APPDATA") { paths.push(format!("{appdata}\\npm")); }
        if let Ok(pf) = std::env::var("ProgramFiles") { paths.push(format!("{pf}\\nodejs")); }
        paths.push(home.join("scoop/shims").display().to_string());
        paths.push(home.join(".volta/bin").display().to_string());
    }

    if let Ok(sys) = std::env::var("PATH") { paths.push(sys); }
    paths.join(if cfg!(windows) { ";" } else { ":" })
}
```

### Unified Binary Finder

```rust
fn find_binary(name: &str, extra_paths: &[PathBuf]) -> Option<PathBuf> {
    let exe = format!("{}{}", name, std::env::consts::EXE_SUFFIX);

    // 1. Check platform-specific known paths
    for dir in extra_paths {
        let c = dir.join(&exe);
        if c.exists() { return Some(c); }
    }
    // 2. Cross-platform paths
    if let Some(home) = dirs::home_dir() {
        for dir in [home.join(".cargo/bin"), home.join(".local/bin")] {
            let c = dir.join(&exe);
            if c.exists() { return Some(c); }
        }
    }
    // 3. Windows: check .cmd wrappers (npm global installs)
    #[cfg(target_os = "windows")]
    for dir in extra_paths {
        let c = dir.join(format!("{name}.cmd"));
        if c.exists() { return Some(c); }
    }
    // 4. Fallback: which/where with expanded PATH
    let cmd = if cfg!(windows) { "where.exe" } else { "which" };
    Command::new(cmd).arg(name).env("PATH", get_full_path()).output().ok()
        .filter(|o| o.status.success())
        .and_then(|o| {
            let p = PathBuf::from(String::from_utf8_lossy(&o.stdout).trim());
            p.exists().then_some(p)
        })
}
```

### Per-Binary Path Tables

**`ck` (CandleKeep CLI):**

| Platform | Known Paths | Install Method |
|----------|-------------|----------------|
| macOS | `/opt/homebrew/bin/ck`, `/usr/local/bin/ck`, `~/.cargo/bin/ck` | `brew install` via Homebrew tap |
| Linux | `~/.local/bin/ck`, `~/.cargo/bin/ck`, `/usr/local/bin/ck` | Download from GitHub Releases → `~/.local/bin/ck` |
| Windows | `%LOCALAPPDATA%\Programs\candlekeep\ck.exe`, `~\.cargo\bin\ck.exe` | Download from GitHub Releases → `%LOCALAPPDATA%\Programs\candlekeep\ck.exe` |

GitHub Releases archives:
- Linux: `ck-x86_64-unknown-linux-gnu.tar.gz`
- Windows: `ck-x86_64-pc-windows-msvc.zip`

**`claude` (Claude Code — npm global):**

| Platform | Known Paths |
|----------|-------------|
| macOS | `/opt/homebrew/bin/claude`, `/usr/local/bin/claude`, `~/.local/bin/claude` |
| Linux | `/usr/local/bin/claude`, `~/.local/bin/claude`, `~/.npm-global/bin/claude`, `~/.nvm/versions/node/*/bin/claude`, `~/.volta/bin/claude` |
| Windows | `%APPDATA%\npm\claude.cmd`, `%APPDATA%\npm\claude`, `%PROGRAMFILES%\nodejs\claude.cmd`, `~\.volta\bin\claude.exe` |

**`node`:**

| Platform | Known Paths |
|----------|-------------|
| macOS | `/opt/homebrew/bin/node`, `/usr/local/bin/node` |
| Linux | `/usr/bin/node`, `/usr/local/bin/node`, `~/.nvm/versions/node/*/bin/node`, `~/.volta/bin/node`, `/snap/bin/node` |
| Windows | `%PROGRAMFILES%\nodejs\node.exe`, `~\.volta\bin\node.exe`, `~\scoop\shims\node.exe` |

**`cargo`:** `~/.cargo/bin/cargo[.exe]` on all platforms (+ `/opt/homebrew/bin/cargo` on macOS).

---

## CK CLI Install via GitHub Releases (platform/installer.rs)

For Linux/Windows (macOS keeps Homebrew):

```rust
async fn install_cli_from_github(platform: &PlatformInfo) -> ActionResult {
    // 1. GET https://api.github.com/repos/CandleKeepAgents/candlekeep-cloud/releases
    //    Filter for latest cli-v* tag
    // 2. Find asset matching platform target triple
    // 3. Download to temp dir
    // 4. Extract: tar.gz (Linux) or zip (Windows)
    // 5. Move binary to platform.paths.cli_install_dir
    // 6. Linux: chmod +x
    // 7. Windows: add cli_install_dir to user PATH via registry if needed
}
```

Deps: `reqwest` (already used for API), `zip` crate, `flate2` + `tar`.

---

## Updater: Replace Custom DMG Flow with Tauri Updater

Current `updater.rs` is a custom DMG-only implementation. Replace with **Tauri's built-in cross-platform updater**:

- Publish update manifest JSON alongside releases
- Tauri handles download, verification, and installation per platform
- Falls back to release URL if in-app update not supported (e.g., some Linux package types)
- Delete `updater.rs` entirely once migrated

---

## App Lifecycle & Tray (lib.rs)

| Item | Change |
|------|--------|
| `set_activation_policy(Accessory)` | Already `#[cfg(target_os = "macos")]` ✅ |
| `icon_as_template(true)` | Wrap in `#[cfg(target_os = "macos")]` |
| `/tmp` fallback | Use `std::env::temp_dir()` |
| Linux tray | Try tray first; if creation fails, fall back to normal visible window |

---

## Frontend Changes

### Setup Wizard (`src/pages/Setup.tsx`)
- **First screen**: Host picker (Claude, Cursor, Codex, Amp)
- **Then**: Platform-aware prerequisite steps
  - macOS: Homebrew → CK CLI via brew → host-specific
  - Linux/Windows: Download CK CLI binary → host-specific
  - Xcode/Homebrew steps hidden on non-macOS

### Integrations Page (`src/pages/Integrations.tsx`)
- Data-driven from `list_integrations()` instead of static cards
- Each card: install/update/repair actions, requirement warnings, OS-specific messaging

### Dashboard (`src/pages/Dashboard.tsx`)
- Status cards consume same generic `IntegrationStatus` — single source of truth

### New Tauri command wrappers (`src/lib/tauri-commands.ts`)
- Add wrappers for new generic API
- Keep old Claude wrappers as shims during migration

---

## Tauri Config (`tauri.conf.json`)

```json
"bundle": {
  "targets": ["dmg", "app", "nsis", "deb", "appimage"],
  "macOS": { "minimumSystemVersion": "10.15" },
  "windows": { "nsis": { "oneClick": true } },
  "linux": { "deb": { "depends": ["libwebkit2gtk-4.1-0", "libappindicator3-1"] } }
}
```

---

## CI/CD (`release.yml`)

Three-platform matrix:

```yaml
strategy:
  matrix:
    include:
      - { os: macos-latest, target: universal-apple-darwin }
      - { os: ubuntu-22.04, target: x86_64-unknown-linux-gnu }
      - { os: windows-latest, target: x86_64-pc-windows-msvc }
```

Linux deps: `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf`

Release must fail if any platform artifact is missing.

---

## Host-Specific Install Strategies

| Host | Install Surface | Notes |
|------|----------------|-------|
| Claude Code | `claude /plugin marketplace add ...` + `claude /plugin install ...` | Existing flow, move behind adapter |
| Cursor | MCP config install (one-click) | NOT a Marketplace plugin for v1 |
| Codex | MCP server registration via config | Via Codex's supported MCP/config flow |
| Amp | MCP server registration via config | Via Amp's supported MCP/config flow |

---

## Implementation Order

### Phase 1: Platform Layer
1. Create `platform/` module (`mod.rs`, `paths.rs`) with `PlatformInfo`, `get_full_path()`, `find_binary()`
2. Migrate `system.rs` functions to use `platform/paths.rs`
3. Create `platform/installer.rs` — GitHub Releases download flow for CK CLI
4. Update `commands/cli.rs` to use platform layer for detection + install

### Phase 2: Integration Layer
5. Define `HostKind`, `IntegrationStatus`, `ActionResult` types
6. Create `Integration` trait + `claude_code.rs` adapter (extract from `plugin.rs`)
7. Create `cursor.rs`, `codex.rs`, `amp.rs` adapters (MCP config install)
8. Register new generic Tauri commands

### Phase 3: Frontend Migration
9. Add TS types + command wrappers for new API
10. Refactor Setup wizard: host picker → platform-aware prerequisites
11. Refactor Integrations page: data-driven from `list_integrations()`
12. Update Dashboard status cards

### Phase 4: Build & Distribution
13. Update `tauri.conf.json` bundle targets
14. Replace custom updater with Tauri updater
15. Expand CI/CD to 3-platform matrix
16. `lib.rs` guards: tray template, temp dir, Linux tray fallback

---

## Files Modified

| File | Change |
|------|--------|
| `src-tauri/src/platform/mod.rs` | **NEW** — Platform enum, PlatformInfo |
| `src-tauri/src/platform/paths.rs` | **NEW** — PATH expansion, find_binary() |
| `src-tauri/src/platform/installer.rs` | **NEW** — GitHub Releases download |
| `src-tauri/src/platform/tray.rs` | **NEW** — Tray + Linux fallback |
| `src-tauri/src/commands/system.rs` | Delegate to platform module |
| `src-tauri/src/commands/cli.rs` | Use platform layer for detection + install |
| `src-tauri/src/commands/plugin.rs` | Extract into integration adapter |
| `src-tauri/src/integrations/mod.rs` | **REWRITE** — trait + manager |
| `src-tauri/src/integrations/claude_code.rs` | **REWRITE** — adapter impl |
| `src-tauri/src/integrations/cursor.rs` | **REWRITE** — MCP config install |
| `src-tauri/src/integrations/codex.rs` | **REWRITE** — MCP config install |
| `src-tauri/src/integrations/amp.rs` | **NEW** — MCP config install |
| `src-tauri/src/updater.rs` | **DELETE** — replace with Tauri updater |
| `src-tauri/src/lib.rs` | Register new commands, `#[cfg]` guards, tray fallback |
| `src-tauri/Cargo.toml` | Add `zip`, `flate2`, `tar` deps |
| `src-tauri/tauri.conf.json` | Multi-platform bundle targets |
| `src/lib/tauri-commands.ts` | New generic API wrappers |
| `src/lib/types.ts` | New shared types |
| `src/pages/Setup.tsx` | Host picker + platform-aware flow |
| `src/pages/Integrations.tsx` | Data-driven from list_integrations() |
| `src/pages/Dashboard.tsx` | Generic integration status |
| `.github/workflows/release.yml` | 3-platform matrix |

---

## Verification

1. `cargo check --manifest-path src-tauri/Cargo.toml` passes on all targets
2. `pnpm exec tsc --noEmit` passes
3. `pnpm tauri build` produces artifacts: DMG (macOS), NSIS (Windows), AppImage+deb (Linux)
4. Binary detection finds `ck`/`claude` in non-standard locations on each platform
5. `install_cli_from_github()` downloads and extracts correct platform binary
6. `list_integrations()` returns status for all 4 hosts
7. Setup wizard shows platform-appropriate steps
8. Linux tray fallback activates when tray unavailable

---

## Open Questions

1. **Windows code signing**: Unsigned .exe/.msi triggers SmartScreen warnings. Sign for v1?
2. **Linux DE support**: Tray varies wildly — test GNOME, KDE, minimal WMs
3. **Cursor/Codex/Amp MCP config paths**: Need to verify exact config file locations per host per platform
