# Arquitectura del Proyecto

## Visión General del Sistema

```
┌────────────────────────────────────────────────────────────────┐
│                  Terminal del Usuario                           │
│  ┌────────────────────────────┐  ┌────────────────────────────┐ │
│  │        tpt-tui             │  │        tpt-cli             │ │
│  │  Dashboard Ratatui         │  │  control · report · import │ │
│  │  (usuario regular)         │  │  (usuario regular)         │ │
│  └───────────┬────────────────┘  └─────────────┬──────────────┘ │
│              │          UDS (JSON, 0600)       │                │
│              └───────────────┬─────────────────┘                │
│                              ▼                                 │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │              tpt-daemon (background, systemd)              │ │
│  │  Event loop: Hyprland socket2 · /proc · idle · reglas      │ │
│  │  Motor de relojes · enforcement (blocklist/penalización)   │ │
│  │  SQLite (WAL) · hilo escritor MPSC · mantenimiento        │ │
│  └───────────────────────────────┬───────────────────────────┘ │
│                                  ▼                             │
│         Adaptadores OS: Hyprland IPC · /proc · idle · BTRFS    │
└────────────────────────────────────────────────────────────────┘
```

**¿Por qué esta separación?** El daemon corre siempre en segundo plano y es el único que toca el OS (captura, persistencia, enforcement). La TUI y el CLI son clientes on-demand por UDS; abrirlos/cerrarlos no interrumpe el tracking. Los adaptadores OS viven detrás de traits para que el core sea testeable sin Wayland/X11.

---

## Estructura de Carpetas

> Estructura **OBJETIVO**. Los crates se crean en Fase 0 (ver `docs/phases.md` y `docs/handoff.md`).

```
tp-cro/
├── AGENTS.md                          # Contexto estático para agentes IA
├── README.md                          # Portafolio (inglés)
├── Cargo.toml                         # Workspace manifest (Fase 0)
├── config.toml                        # Configuración por defecto/ejemplo
├── .gitignore
│
├── crates/
│   ├── tpt-core/                      # Dominio puro: modelos, puertos (traits), reglas
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── config/                # Modelos de configuración + parsing TOML
│   │   │   ├── domain/                # Categorías, tareas, sesiones, estados
│   │   │   ├── ports/                 # WindowSource, IdleSource, Clock, Store, ...
│   │   │   ├── rules/                 # Motor de reglas regex + categorizador
│   │   │   ├── timers/                # State machine Pomodoro/Focus/Flowtime
│   │   │   └── analytics/             # Agregados y métricas
│   │   └── tests/
│   │
│   ├── tpt-daemon/                    # Adaptadores OS + event loop
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── adapters/              # Hyprland, /proc, idle, sqlite, toml, clock
│   │   │   ├── engine.rs              # Orquestación captura → categoriza → persiste
│   │   │   └── ipc_server.rs          # Servidor UDS
│   │   └── tests/
│   │
│   ├── tpt-tui/                       # Interfaz Ratatui
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── app.rs                 # Orquestación
│   │   │   ├── views/                 # Dashboard, timer, analytics, modales
│   │   │   └── ipc_client.rs
│   │   └── tests/
│   │
│   └── tpt-cli/                       # Cliente de control/report/import (D1)
│       ├── src/
│       │   ├── main.rs
│       │   ├── commands/              # status, focus, pomodoro, report, import
│       │   └── ipc_client.rs
│       └── tests/
│
├── docs/                              # Documentación (ver docs/index.md)
├── .github/workflows/ci.yml           # CI (Fase 0)
└── config/                            # (Fase 10) unit systemd, PKGBUILD
```

---

## Crate `tpt-core` — Puertos (traits)

Hexagonal: el dominio NO depende del OS. Los adaptadores los implementan en `tpt-daemon`.

| Puerto | Responsabilidad | Adaptador (daemon) |
|--------|-----------------|--------------------|
| `WindowSource` | Evento de ventana activa (class, title, pid, pwd) | `HyprlandWindowSource` |
| `IdleSource` | Señal de inactividad input | `HyprlandIdleSource` (sin libwayland) + fallback |
| `Clock` | Tiempo monotónico y wall-clock | `SystemClock` |
| `Store` | Persistencia (logs, sesiones, agregados, retención) | `SqliteStore` |
| `ConfigSource` | Carga/validación de configuración | `TomlConfig` |
| `Notifier` | Notificaciones del daemon (fatiga, avisos) | `NotifySender` |
| `IpcTransport` | Transporte UDS compartido | `UnixSocketTransport` |

---

## Dependencias planificadas (Cargo)

| Crate | Dependencias |
|-------|--------------|
| `tpt-core` | `serde`, `serde_json`, `toml`, `regex`, `thiserror`, `rusqlite` (bundled) |
| `tpt-daemon` | `tpt-core`, `serde_json`, `libc` (proc), `nix` (socket) |
| `tpt-tui` | `tpt-core`, `ratatui`, `crossterm`, `serde_json` |
| `tpt-cli` | `tpt-core`, `serde_json`, `clap` |

---

## Protocolo IPC

Comunicación via **Unix domain socket** en `$XDG_RUNTIME_DIR/tpt.sock`, permisos `0600` (D4). Mensajes JSON con framing de longitud. Protocolo versionado (`protocol_version` + `min_supported` en el handshake).

```
TUI/CLI (cliente)                     Daemon (servidor)
    │                                      │
    ├─── {"v":1,"type":"status"} ────────▶│
    │◀── {"v":1,"type":"status_response", │
    │      "timer":{...},"cooldown":300} ──┤
    │                                      │
    ├─── {"v":1,"type":"focus_start",     │
    │      "mode":"FOCUS","minutes":45} ──▶│
    │◀── {"v":1,"type":"ok"} ────────────┤
```

Tipos principales: `status`, `status_response`, `focus_start`, `focus_abort`, `timer_adjust`, `task_switch`, `pomodoro_*`, `flowtime_*`, `blocklist_event`, `ok`, `error`.

---

## Esquema SQLite

Esquema completo (categorías, tareas, blocklist por tarea, logs, sesiones, agregados) → `docs/adr/ADR-002-sqlite-schema.md`.

Resumen:
- `categories` — categorías + `is_productive`.
- `recurring_tasks` — rutinas con meta de minutos.
- `task_blocked_apps` — **blocklist por tarea** (D13).
- `window_activity_logs` — logs crudos de actividad (raw, retención corta).
- `focus_sessions` — sesiones con modo, productivo vs observado, reflexión, causa de aborto.
- `daily_aggregates` / `hourly_aggregates` — agregados para analíticas y retención modular (D17).

---

## Persistencia y BTRFS (D18)

- DB en `~/.local/share/tpt/metrics.db`, WAL mode.
- `chattr +C` (nodatacow) en el directorio de la DB **antes** de crearla.
- Checkpoint `wal_checkpoint(TRUNCATE)` periódico.
- `auto_vacuum=INCREMENTAL` + `incremental_vacuum(N)`; nunca `VACUUM` completo.
- Snapshots BTRFS: excluir la DB o usar backup consistente (API `.backup` / `VACUUM INTO`).
- Retención por tiers configurable en `config.toml` (ver `ADR-006-data-retention-btrfs.md`).

---

## Servicio systemd (Fase 10)

```ini
# config/tpt-daemon.service
[Unit]
Description=Terminal Productivity Tracker Daemon
After=graphical-session.target

[Service]
Type=simple
ExecStart=/usr/local/bin/tpt-daemon
Restart=always
RestartSec=5

RuntimeDirectory=tpt
RuntimeDirectoryMode=0700

StandardOutput=journal
StandardError=journal
SyslogIdentifier=tpt

[Install]
WantedBy=default.target
```

---

## Decisiones de Arquitectura

Resumen de decisiones — el detalle vive en `docs/adr/`.

| Decisión | ADR |
|----------|-----|
| Arquitectura general, persistencia e IPC | `ADR-001-architecture-ipc-persistence.md` |
| Esquema SQLite | `ADR-002-sqlite-schema.md` |
| Puertos/adaptadores (hexagonal) | `ADR-003-hexagonal-ports.md` |
| Modos de reloj y penalizaciones | `ADR-004-clock-modes-and-penalties.md` |
| Inactividad y blocklist | `ADR-005-inactivity-and-blocklist.md` |
| Datos y BTRFS | `ADR-006-data-retention-btrfs.md` |
| Crate `tpt-cli` | `ADR-007-tpt-cli-crate.md` |