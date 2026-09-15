# TPT — Terminal Productivity Tracker

> Passive time, focus and fatigue tracker for Linux power users — 100% local, terminal-first, low memory footprint.

![Rust](https://img.shields.io/badge/Rust-1.85+-orange) ![License](https://img.shields.io/badge/license-MIT-green)

## Problem it solves

Staying focused on a Linux desktop is hard. TPT passively tracks which windows you use, which project you are in (`$PWD`), and how long you actually stay focused — then helps you commit to it with three timer modes (Pomodoro, Focus contract, Flowtime), per-task app blocklists, and honest analytics. All data stays on your machine. No telemetry, no cloud, no screenshots, no keylogging.

## Features

- **Passive tracking** via Hyprland IPC (`socket2.sock`) + terminal `$PWD` (`/proc/<pid>/cwd`)
- **Rule-based categorization** (regex on class/title/pwd, no AI) into productive/distraction categories
- **Three timer modes**: Pomodoro (no pause, cycle reset on abort), Focus contract (immutable, penalized abort), Flowtime (measure, don't impose)
- **Editable presets**: `25/5`, `50/10`, `52/17`, `90/15` (long break 20) — short break ≤15 min, long ≤20 min
- **Per-task app blocklist**: forbidden apps per task; opening one triggers warning → countdown → penalty per mode; window switching is free
- **Smart inactivity**: no-keyboard never penalizes; informative warning after 300 s; idle gaps classified as offline (never productive)
- **Abort flow**: mandatory reflection + challenge/password in both modes; Focus abort = credit 0 + 5 min cooldown (visible in TUI)
- **Routines & streaks**: recurring daily tasks with progress bars, monthly habit consistency
- **Persistent analytics**: day/week/month views, hours invested, worked days, averages (per worked day and per calendar day), productive vs distraction
- **CLI reporting & import**: `tpt-cli report --range weekly --format json`, `tpt-cli import --super-productivity save.json`
- **BTRFS-friendly storage**: nodatacow, WAL checkpoints, modular retention tiers (raw/hourly/daily)

## Stack and why

| Layer | Technology | Why |
|---|---|---|
| Language | Rust | Zero-overhead, no async runtime needed, memory budget friendly |
| TUI | ratatui + crossterm | Mature terminal UI, event-driven |
| IPC | Unix domain socket + length-framed JSON | Fast, dependency-free, local-only, versioned protocol |
| Persistence | rusqlite (WAL) | Simple, fast, concurrent reads, single writer thread |
| Config | TOML (`config.toml`) | Human-readable, defaults-first |
| Errors | thiserror | No panics in production code |

## Architecture

```
┌──────────────────────────────────────────────────────────┐
│  tpt-tui (user)            tpt-cli (user)                │
│  ratatui dashboard   ◄────►  control/report/import       │
│      │                       │                           │
│      └────────── UDS (versioned JSON, 0600) ────────────┘
│                         ▼                                │
│  tpt-daemon (background, systemd)                        │
│  Hyprland IPC · /proc · idle · rule engine · SQLite      │
│  Ports: WindowSource, IdleSource, Clock, Store, ...      │
└──────────────────────────────────────────────────────────┘
```

Hexagonal: domain ports live in `tpt-core`; OS adapters live in `tpt-daemon`. The daemon runs always; TUI/CLI are on-demand clients.

## Setup

**Requirements:** Rust stable, Arch Linux (Wayland/Hyprland), systemd.

```bash
git clone <repo-url>
cd tp-cro

cargo build
cargo test
```

## Usage

```bash
# Daemon (background)
sudo systemctl start tpt-daemon

# TUI dashboard
cargo run -p tpt-tui

# CLI control / reports / import
cargo run -p tpt-cli -- status
cargo run -p tpt-cli -- focus start --minutes 45
cargo run -p tpt-cli -- report --range weekly --format json
cargo run -p tpt-cli -- import --super-productivity ~/.config/superProductivity/save.json
```

## Testing

- Unit: `cargo test` — domain rules, timer state machine, categorizer, config round-trip
- Integration: adapters with fakes + temporary SQLite, IPC socket, idle flows
- Functional: daemon lifecycle on a real Hyprland session (optional in CI)

## Key Decisions

- **Daemon / TUI / CLI split**: the daemon owns capture and persistence; TUI and CLI are thin IPC clients. Keeps RAM low and processes independently restartable.
- **Idle is a fact, not an infraction**: no-keyboard never penalizes metrics; only opening a blocked app can trigger a penalty.
- **Focus = hard commitment**: fixed, immutable contract; abort costs credit + cooldown + reflection (loss aversion by design).
- **BTRFS-aware storage**: nodatacow DB directory, checkpointed WAL, incremental vacuum, retention tiers.

## Roadmap

- [x] Phase 0 — Workspace & CI
- [ ] Phase 1 — Config & domain ports
- [ ] Phase 2 — SQLite store
- [ ] Phase 3 — Capture & categorization
- [ ] Phase 4 — Inactivity & blocklist enforcement
- [ ] Phase 5 — Timer engine (Pomodoro/Focus/Flowtime)
- [ ] Phase 6 — IPC protocol
- [ ] Phase 7 — TUI dashboard
- [ ] Phase 8 — Analytics & reporting
- [ ] Phase 9 — Routines, streaks & Super Productivity import
- [ ] Phase 10 — Ops, packaging & polish

## License

MIT — 2026