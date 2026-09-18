# Arquitectura del Proyecto

## Visión General del Sistema

```
┌────────────────────────────────────────────────────────────────┐
│                  Terminal del Usuario                           │
│  ┌────────────────────────────┐  ┌────────────────────────────┐ │
│  │        tpt-tui             │  │        tpt-cli             │ │
│  │  Hoy · Timer · Métricas    │  │  control · report · import │ │
│  │  Actividades · Historial   │  │  daemon lifecycle          │ │
│  └───────────┬────────────────┘  └─────────────┬──────────────┘ │
│              │          UDS (JSON, 0600)       │                │
│              └───────────────┬─────────────────┘                │
│                              ▼                                 │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │              tpt-daemon (background, systemd)              │ │
│  │  Motor de sesiones · contrato · penalizaciones             │ │
│  │  Presencia (quickshell hooks + salto de reloj)             │ │
│  │  SQLite (WAL) · hilo escritor MPSC · mantenimiento         │ │
│  └───────────────────────────────┬───────────────────────────┘ │
│                                  ▼                             │
│         Adaptadores: quickshell hook · clock · sqlite · toml    │
└────────────────────────────────────────────────────────────────┘
```

**¿Por qué esta separación?** El daemon es dueño de la **sesión**, no solo de la persistencia. Si el timer viviera en la TUI, cerrarla (`q`) sería la forma obvia de escapar del compromiso. Con el daemon, la sesión sigue viva, sigue contando y abortar cuesta. **El daemon es la restricción.**

La TUI y el CLI son clientes on-demand por UDS; abrirlos/cerrarlos no interrumpe nada. Los adaptadores de OS viven detrás de traits para que el core sea testeable sin Hyprland ni Wayland.

---

## Estructura de Carpetas

> Estructura creada en Fase 0. Los módulos internos se agregan por fase.

```
tp-cro/
├── AGENTS.md                          # Contexto estático para agentes IA
├── README.md                          # Portafolio (inglés)
├── Cargo.toml                         # Workspace manifest (edition 2024, resolver 3)
├── config.toml                        # Configuración por defecto/ejemplo
├── .gitignore
│
├── crates/
│   ├── tpt-core/                      # Dominio puro: modelos, puertos, contrato
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── config/                # Modelos de configuración + parsing TOML
│   │   │   ├── domain/                # Activity, Project, Tag, Schedule, TimeEntry, Session
│   │   │   ├── ports/                 # Clock, Store, ConfigSource, PresenceSource, Notifier, IpcTransport
│   │   │   ├── timers/                # Máquina de sesiones: modos, transiciones, contrato
│   │   │   ├── presence/              # Política de inactividad, niveles, escalada
│   │   │   └── analytics/             # Métricas, agregaciones, rachas
│   │   └── tests/
│   │
│   ├── tpt-daemon/                    # Adaptadores + event loop
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── adapters/              # quickshell hook, sqlite, toml, clock, notifier
│   │   │   ├── engine.rs              # Orquestación sesión → presencia → persistencia
│   │   │   └── ipc_server.rs          # Servidor UDS
│   │   └── tests/
│   │
│   ├── tpt-tui/                       # Interfaz Ratatui
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── app.rs                 # Orquestación
│   │   │   ├── views/                 # Hoy, Timer, Métricas, Actividades, Historial, Config
│   │   │   └── ipc_client.rs
│   │   └── tests/
│   │
│   └── tpt-cli/                       # Cliente de control/report/import (D1)
│       ├── src/
│       │   ├── main.rs
│       │   ├── commands/              # status, session, activity, report, import, daemon
│       │   └── ipc_client.rs
│       └── tests/
│
├── docs/                              # Documentación (ver docs/index.md)
└── .github/workflows/ci.yml           # CI (fmt + clippy + test)
```

---

## Crate `tpt-core` — Puertos (traits)

Hexagonal: el dominio NO depende del OS (D2).

| Puerto | Responsabilidad | Adaptador (daemon) |
|--------|-----------------|--------------------|
| `Clock` | Monotónico + wall-clock; detección de salto | `SystemClock` |
| `PresenceSource` | Idle / active / screen-off / suspended / resumed | `QuickshellHookSource` + `ClockGapSource` |
| `Store` | Persistencia (actividades, entradas, sesiones, notas) | `SqliteStore` |
| `ConfigSource` | Carga/validación de configuración | `TomlConfig` |
| `Notifier` | Avisos del daemon (escalada, fin de bloque) | `NotifySender` |
| `IpcTransport` | Transporte UDS compartido | `UnixSocketTransport` |

Detalle del cambio de puertos → `ADR-003-hexagonal-ports.md`.

---

## Dependencias planificadas (Cargo)

| Crate | Dependencias |
|-------|--------------|
| `tpt-core` | `serde`, `serde_json`, `toml`, `thiserror` |
| `tpt-daemon` | `tpt-core`, `serde_json`, `rusqlite` (bundled), `nix` |
| `tpt-tui` | `tpt-core`, `ratatui`, `crossterm`, `serde_json` |
| `tpt-cli` | `tpt-core`, `serde_json`, `clap` |

> **Regla:** ninguna dependencia entra sin justificar su costo en memoria (ver `AGENTS.md`). No se usa tokio ni ningún runtime async.

---

## Protocolo IPC

Comunicación via **Unix domain socket** en `$XDG_RUNTIME_DIR/tpt.sock`, permisos `0600` (D4). Mensajes JSON con framing de longitud. Protocolo versionado (`protocol_version` + `min_supported` en el handshake).

```
TUI/CLI (cliente)                        Daemon (servidor)
    │                                        │
    ├─── {"v":1,"type":"status"} ───────────▶│
    │◀── {"v":1,"type":"status_response",    │
    │      "session":{...},"remaining":312}──┤
    │                                        │
    ├─── {"v":1,"type":"session_start",      │
    │      "activity_id":7,"mode":"FOCUS"}──▶│
    │◀── {"v":1,"type":"ok"} ────────────────┤
```

Tipos principales: `status`, `status_response`, `session_start`, `session_extend`, `session_abort`, `session_switch_activity`, `break_skip`, `presence_event`, `activity_*`, `notes_list`, `daemon_stop`, `ok`, `error`.

---

## Modelo de Datos

Esquema SQLite completo → `ADR-002-sqlite-schema.md`.

Resumen: `projects`, `activities`, `tags`, `activity_tags`, `time_entries` (manual/session/imported), `sessions`, `session_additions`, `session_gaps`, `commitment_contracts`.

Sin tablas de agregados ni tiers de retención: las métricas se calculan con `GROUP BY` sobre `sessions` + `time_entries` (`ADR-002`).

---

## Presencia (idle, pantalla apagada, power)

Fuente primaria: **hooks de `caelestia-shell`** (quickshell). El shell crea un `IdleMonitor` (`ext-idle-notify-v1`) por cada entrada de `general.idle.timeouts` en `~/.config/caelestia/shell.json`; agregamos entradas propias que ejecutan `tpt-cli presence --state <s>` contra el socket del daemon. El **bloqueo de pantalla se ignora**; la señal severa es la **pantalla apagada** (`dpms off`).
Backstop: **salto de reloj** (monotónico vs wall-clock) para suspensión/apagado.

Consecuencia: **cero dependencias nuevas**, nada de D-Bus ni libwayland. Argumentación completa → `ADR-008-presence-and-penalties.md`.

---

## Persistencia y BTRFS

- DB en `${XDG_DATA_HOME:-~/.local/share}/tpt/metrics.db`, WAL mode.
- **BTRFS:** `chattr +C` (nodatacow) en el directorio de la DB **antes** de crearla; sin esto hay write amplification por CoW.
- Checkpoint `wal_checkpoint(TRUNCATE)` periódico; `auto_vacuum=INCREMENTAL` + `incremental_vacuum`.
- **Nunca** `VACUUM` completo (reescribe todo el archivo; pésimo en CoW).
- Snapshots BTRFS: excluir la DB o usar backup consistente (`VACUUM INTO`).

---

## Estrategia de Testing

```
        ╱╲
       ╱  ╲      Funcional   — daemon real + Hyprland (opcional en CI)
      ╱────╲
     ╱      ╲    Integración — adaptadores con fakes + SQLite temporal
    ╱────────╲               — cliente/servidor IPC real (socket UDS)
   ╱          ╲
  ╱────────────╲ Unitarios  — tpt-core `#[cfg(test)]`: contrato, presencia,
 ────────────────             config round-trip, métricas
```

**Reglas:** test ANTES de implementar (RED → GREEN → REFACTOR). Esfera de tests por capa (la de arriba se construye sobre la de abajo). Nada de lógica en `tpt-daemon` que no esté cubierta por un fake.

---

## CI (GitHub Actions)

Implementado en `.github/workflows/ci.yml`. Corre en cada push y PR, con caché de cargo:

1. `cargo fmt --all --check`
2. `cargo clippy --workspace --all-targets --locked -- -D warnings`
3. `cargo test --workspace --all-targets --locked`

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

Resumen — el detalle vive en `docs/adr/`.

| Decisión | ADR |
|----------|-----|
| Arquitectura general, persistencia e IPC | `ADR-001-architecture-ipc-persistence.md` |
| Esquema SQLite | `ADR-002-sqlite-schema.md` |
| Puertos/adaptadores (hexagonal) | `ADR-003-hexagonal-ports.md` |
| Modos de reloj, contrato y penalizaciones | `ADR-004-clock-modes-and-penalties.md` |
| Crate `tpt-cli` | `ADR-005-tpt-cli-crate.md` |
| Branching y protección de ramas | `ADR-006-branching-and-protection.md` |
| Pivote a *timer-first* | `ADR-007-timer-first-pivot.md` |
| Presencia: idle, pantalla apagada y power | `ADR-008-presence-and-penalties.md` |
