# Plan de Desarrollo

Estrategia de testing, CI, packaging y deployment.

---

## Estrategia de Testing — Pirámide

```
        ╱╲
       ╱  ╲      Funcional
      ╱    ╲     — Ciclo de vida del daemon real (Hyprland, opcional en CI)
     ╱──────╲    — Flujos E2E: captura → categoriza → persiste → reporta
    ╱        ╲
   ╱──────────╲  Integración
  ╱            ╲ — Adaptadores con fakes + SQLite temporal (tempfile)
 ╱              ╲ — Cliente/servidor IPC real (socket UDS)
╱────────────────╲— Operaciones Store (WAL, retención, agregados)
╱                  ╲
─────────────────── Unitarios (tpt-core, `#[cfg(test)]`)
                    — Motor de reglas, state machine de relojes, config round-trip
                    — Muchos, rápidos, aislados
```

---

## Ciclo TDD por Feature

```
┌─────────────────────────────────────────────────────────┐
│                    CICLO TDD                              │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  1. RED ──────── Escribir test que falla                 │
│       │         (define comportamiento esperado)         │
│       ▼                                                  │
│  2. GREEN ────── Escribir código mínimo para pasar       │
│       │         (implementar funcionalidad)              │
│       ▼                                                  │
│  3. REFACTOR ─── Limpiar código sin cambiar comportamiento│
│       │         (clippy --all-targets -- -D warnings)    │
│       ▼                                                  │
│  4. INTEGRACIÓN ── Test de integración si toca límites   │
│       │           externos (SQLite, IPC, /proc fake)     │
│       ▼                                                  │
│  5. FUNCIONAL ── Test E2E del daemon real (opcional CI)  │
│       │                                                  │
│       ▼                                                  │
│  6. REPETIR ──── Volver al paso 1 con siguiente feature  │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

**Reglas:**
- ANTES de escribir implementación → escribir el test.
- Si el test falla → está bien, es RED.
- Escribir solo el código necesario para que pase → GREEN.
- Una vez que pasa → refactorizar.
- NUNCA escribir implementación sin test previo.

---

## Convención de Nombres de Tests

```rust
// Patrón: fn <método>_<condición>_<esperado>()
#[cfg(test)]
mod tests {
    #[test]
    fn rule_engine_matches_class_returns_category() { /* ... */ }

    #[test]
    fn focus_session_abort_penalizes_credit_to_zero() { /* ... */ }
}
```

---

## Organización de Archivos de Test

```
crates/
├── tpt-core/
│   ├── src/
│   │   ├── rules/reglas.rs        # + #[cfg(test)] unit tests en módulo
│   │   ├── timers/state.rs        # + #[cfg(test)] unit tests en módulo
│   │   └── ...
│   └── tests/                     # integración (multi-módulo)
│       ├── store_sqlite_test.rs
│       └── config_roundtrip_test.rs
├── tpt-daemon/
│   └── tests/                     # adaptadores con fakes + IPC real
├── tpt-tui/
│   └── tests/                     # componentes de vista (render con datos fake)
└── tpt-cli/
    └── tests/                     # comandos sobre IPC fake
```

---

## Cuándo Escribir Cada Tipo de Test

| Feature | Unitario | Integración | Funcional |
|---------|----------|-------------|-----------|
| Config TOML round-trip | ✅ | ✅ archivo real | ❌ |
| Motor de reglas regex | ✅ todas las rutas | ❌ | ❌ |
| State machine de relojes | ✅ modos/penalizaciones | ✅ con Store real | ❌ |
| Blocklist / inactividad | ✅ flujos lógicos | ✅ fakes + Store | ✅ daemon |
| Protocolo IPC | ✅ serialización | ✅ socket real | ✅ daemon completo |
| Captura Hyprland | ✅ parsing de eventos | ✅ fake WindowSource | ✅ sesión real |
| Analíticas | ✅ queries | ✅ DB real + agregados | ✅ pipeline completo |
| Import Super Productivity | ✅ parsing | ✅ archivo real + dedup | ❌ |

---

## CI (GitHub Actions)

Implementado en `.github/workflows/ci.yml` (Fase 0). Corre en cada push y PR, con caché de cargo (`Swatinem/rust-cache`):

1. `cargo fmt --all --check`
2. `cargo clippy --workspace --all-targets --locked -- -D warnings`
3. `cargo test --workspace --all-targets --locked`

La misma secuencia se corre local antes de cada commit (ver `AGENTS.md`).

---

## Empaquetado Arch Linux (Fase 10)

- Unit systemd: `config/tpt-daemon.service` (ver `docs/architecture.md`).
- PKGBUILD para `tpt-daemon`, `tpt-tui` y `tpt-cli` (release builds, paths XDG).
- Instalación objetivo: binarios en `/usr/local/bin`, DB en `~/.local/share/tpt/`.

## Checklist de Deployment

```bash
# Build release
cargo build --release

# Instalar daemon
sudo install -m755 target/release/tpt-daemon /usr/local/bin/
sudo install -m644 config/tpt-daemon.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now tpt-daemon

# Instalar TUI y CLI
sudo install -m755 target/release/tpt-tui /usr/local/bin/
sudo install -m755 target/release/tpt-cli /usr/local/bin/

# BTRFS: nodatacow en el directorio de la DB ANTES de crearla
mkdir -p ~/.local/share/tpt
sudo chattr +C ~/.local/share/tpt   # aplicar antes de la primera ejecución

# Iniciar
tpt-tui
tpt-cli status
```

---

## BTRFS y Retención (D17, D18)

- **nodatacow**: `chattr +C` en el directorio de la DB (solo funciona en directorios vacíos o antes de crear el archivo).
- **WAL**: `synchronous=NORMAL` + checkpoint `TRUNCATE` periódico para evitar WAL gigante.
- **Vacuum incremental**: `auto_vacuum=INCREMENTAL` + `PRAGMA incremental_vacuum(N)`; nunca `VACUUM` completo (reescribe todo el archivo).
- **Snapshots**: excluir la DB del snapshot o usar backup consistente (`VACUUM INTO` / API `.backup`).
- **Retención modular** (configurable en `config.toml`):
  - `raw_events` → 30 días (default).
  - `hourly_aggregates` → 90 días (default).
  - `daily_aggregates` → permanente.
- Una tarea de mantenimiento en el daemon ejecuta la retención y los checkpoints en horarios de baja actividad.

---

## Git + PRs

- Commits convencionales en inglés (ver `AGENTS.md`).
- **Commit por feature** al cerrar cada feature TDD.
- **PR por fase** al cumplir los criterios de salida de la fase (log + learning + handoff actualizados).
- Flujo por fase: rama `type/phase-N-...` desde `main`, un commit por feature, PR contra `main` al cerrar la fase.
- Remoto: `origin` → `github.com:ails-w/tpo-cro` (rama `main`).