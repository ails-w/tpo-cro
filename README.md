# TPT — Terminal Productivity Tracker

> Activity and time-contract tracker for Linux power users. 100% local, terminal-first, low memory footprint.

![Rust](https://img.shields.io/badge/Rust-1.85+-orange) ![License](https://img.shields.io/badge/license-MIT-green)

## Problem it solves

Staying focused is not a matter of willpower — it is a matter of **commitment you cannot quietly walk away from**. TPT tracks activities the way Super Productivity does, but wraps them in a session engine that will not let you pause, shrink, or escape a commitment. Every minute is either earned or it isn't. All data stays on your machine: no telemetry, no cloud, no screenshots, no keylogging.

## What it does

- **Activities & projects** — create activities with optional targets, projects, tags and recurrence rules
- **Day queue** — activities scheduled weekdays, specific days, all week, or within a date range
- **Three session modes** — Flowtime (free, pausable), Pomodoro (cycles, adjustable), Focus (hard, immutable contract)
- **Additive contract** — you can only *extend* a session, never shorten it. No pause in Pomodoro or Focus
- **Presence tracking** — idle, screen-off and suspend are detected; absent time is simply not credited
- **Configurable strictness** — `Off` / `L0` / `L1` / `L2`, with a time-boxed commitment that blocks downgrades
- **Manual time** — log work done away from the timer; tagged, and counted in totals
- **Reflection** — optional 1–10 focus rating and notes on any session, browsable in a date-grouped history
- **Metrics** — yearly heatmap, focus-time timeline, year→month→week→day hierarchy, averages per worked day and per calendar day
- **Super Productivity import** — bring your existing tasks and time tracking over, idempotently

### What it deliberately does **not** do

No window tracking. No app categorization. No app blocklist. There is no enforcement over other programs: the contract is **self-imposed**. TPT is the honest stopwatch and the record — not the guard.

## Stack and why

| Layer | Technology | Why |
|---|---|---|
| Language | Rust | Zero-overhead, no async runtime needed, memory budget friendly |
| TUI | ratatui + crossterm | Mature terminal UI, event-driven |
| IPC | Unix domain socket + length-framed JSON | Fast, dependency-free, local-only, versioned protocol |
| Persistence | rusqlite (WAL) | Metrics are the core: SQL aggregation beats hand-rolled code |
| Config | TOML | Human-readable, defaults-first |
| Presence | caelestia-shell (quickshell) idle hooks + clock-gap detection | Zero new dependencies, no D-Bus, no libwayland |
| Errors | thiserror | No panics in production code |

## Architecture

```
┌──────────────────────────────────────────────────────────┐
│  tpt-tui (on demand)       tpt-cli (control/report)      │
│  day queue · timer         import · daemon lifecycle     │
│      │                       │                           │
│      └────────── UDS (versioned JSON, 0600) ────────────┘
│                         ▼                                │
│  tpt-daemon (background, systemd)                        │
│  session engine · contract · presence · SQLite           │
│  Ports: Clock, PresenceSource, Store, ConfigSource, ...  │
└──────────────────────────────────────────────────────────┘
```

Hexagonal: domain ports live in `tpt-core`; OS adapters live in `tpt-daemon`. **The daemon owns the session**, so closing the TUI is not a way out.

## Setup

**Requirements:** Rust stable, Arch Linux (Wayland/Hyprland), systemd, and the `caelestia-shell` (quickshell) reporting presence.

```bash
git clone git@github.com:ails-w/tpo-cro.git
cd tpo-cro

cargo build
cargo test
```

## Usage

```bash
cargo run -p tpt-daemon                    # background session owner
cargo run -p tpt-tui                       # dashboard
cargo run -p tpt-cli -- status
cargo run -p tpt-cli -- session start --activity 3 --mode FOCUS
cargo run -p tpt-cli -- report --range weekly --format json
cargo run -p tpt-cli -- import --super-productivity ~/.config/superProductivity/save.json
```

## Testing

- Unit: domain contract, presence policy, config round-trip, metrics
- Integration: adapters with fakes + temporary SQLite, real IPC socket
- Functional: daemon lifecycle on a real Hyprland session (optional in CI)

## Key Decisions

- **Additive contract**: a session can only grow. You cannot shorten your way out of a commitment.
- **The daemon owns the session**: closing the TUI does not abort. Aborting costs a challenge, a reflection and a cooldown.
- **Default is kind**: L0 detects absence and stops crediting it, but never punishes. Hard penalties are opt-in and time-boxed.
- **Presence without dependencies**: the `caelestia-shell` already solved Wayland idle; we consume its idle hooks and use clock-gap detection as a kernel-level backstop.
- **SQLite over JSON**: metrics are the product, and SQL aggregation is the right tool. JSON is only for import/export.

## Roadmap

- [x] Phase 0 — Workspace & CI
- [ ] Phase 1 — Docs pivot + domain, config and ports
- [ ] Phase 2 — SQLite store
- [ ] Phase 3 — Activities, projects, tags, schedule & queue
- [ ] Phase 4 — Session engine and contract
- [ ] Phase 5 — Presence: idle, screen-off and suspend
- [ ] Phase 6 — Versioned IPC protocol
- [ ] Phase 7 — TUI
- [ ] Phase 8 — Metrics and reports
- [ ] Phase 9 — Streaks and Super Productivity import
- [ ] Phase 10 — Ops, packaging & polish

## License

MIT — 2026
