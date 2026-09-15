# Fase 0: Setup — Log

> Log HISTÓRICO de la fase. Se acumula, no se borra.
> Estado actual → `docs/handoff.md` · Conceptos → `docs/learning/phase-00-setup.md`

## Estado

**Estado**: Completada
**Última Actualización**: 2026-09-14 20:46

## Objetivos

- Dejar lista la infraestructura del workspace Rust con los 4 crates skeleton.
- Dejar el pipeline de CI (fmt + clippy + test) verde.

## Progreso

- [x] Feature 0.1 — Workspace (2026-09-14)
- [x] Feature 0.2 — CI (2026-09-14)

## Tareas Completadas

### 2026-09-14 — Feature 0.1: Workspace

- **Descripción**: Workspace Rust (edition 2024, resolver 3) con 4 crates `lib` + `bin`. Cada crate expone `version()` y un smoke test. Lints de workspace heredados.
- **Archivos**: `Cargo.toml`, `Cargo.lock`, `crates/{tpt-core,tpt-daemon,tpt-tui,tpt-cli}/{Cargo.toml,src/{lib.rs,main.rs}}`
- **Tests**: `tpt_core_smoke_lib_exposes_version`, `tpt_daemon_smoke_lib_exposes_version`, `tpt_tui_smoke_lib_exposes_version`, `tpt_cli_smoke_lib_exposes_version` — 4/4 verdes
- **TDD**: RED (`cannot find function version` en los 4 crates) → GREEN (`version()` implementado) → REFACTOR (fmt + clippy limpios)
- **Commit**: `chore: scaffold cargo workspace with four crates`

### 2026-09-14 — Feature 0.2: CI

- **Descripción**: Workflow de GitHub Actions que corre en push y PR: fmt, clippy y test, con caché de cargo.
- **Archivos**: `.github/workflows/ci.yml`
- **Verificación local**: los 3 pasos del workflow ejecutados a mano → fmt OK, clippy limpio, 4 tests pasando
- **Commit**: `ci: add fmt, clippy and test workflow`

## Decisiones

1. **Edition 2024 + resolver 3** — es lo idiomático con Rust 1.98; el MSRV real sube a 1.85 (badge del README actualizado).
2. **Cada crate es `lib` + `bin`** — `architecture.md` ya prevé módulos (`app.rs`, `views/`, `ipc_client.rs`) y Fase 7 exige tests de componentes; un binario puro no es importable desde `tests/`.
3. **Lints de workspace con `deny` de `unwrap_used`/`expect_used`/`panic`** — traduce a la herramienta la regla de "cero panics" de `AGENTS.md`. Excepción acotada con `#![cfg_attr(test, allow(...))]`.
4. **Sin dependencias de negocio en Fase 0** — `serde`/`toml`/`regex`/`rusqlite`/etc. entran cuando el feature las pida (Fase 1+), para no inflar el build del daemon.
5. **`rust-toolchain.toml` descartado** — no hay `rustup` en el entorno (Rust viene del paquete de Arch); el archivo sería inerte localmente. El CI sí usa `dtolnay/rust-toolchain@stable`.
6. **`Cargo.lock` versionado** — es un proyecto de binarios; el lock se commitea para builds reproducibles (`--locked` en CI).

## Problemas

1. **Sin `rustup`** — los comandos `rustup component add` no existen en este entorno; se resolvió usando el paquete `rust` de Arch (que ya trae `rustfmt` y `clippy`).
2. **`cargo clippy` no recompilaba** — con todo ya compilado, clippy emitía "Finished" sin re-lintear; se forzó un cambio temporal para verificar que el lint de `unwrap_used` realmente dispara (confirmado).

## Métricas

- Tests escritos: 4 (smoke, uno por crate)
- Tests pasando: 100%
- Cobertura: N/A (fase de infraestructura, sin lógica de negocio)

## Pendientes

- Fase 1 — Configuración y dominio: modelos, parsing TOML con defaults, validación de presets (corto ≤15, largo ≤20) y puertos (traits).
