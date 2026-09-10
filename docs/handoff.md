# Handoff — Continuidad

> ⚠️ **ESTADO MUTABLE.** Se SOBREESCRIBE al cerrar sesión. No es historial.
> Historial por fase → `docs/progress-log/`. Conceptos → `docs/learning/`.
> Aprendizajes y decisiones persistentes → Engram (memoria).

**Última actualización**: 2026-09-10

## Estado actual

| | |
|---|---|
| **Fase activa** | Fase 0 — Setup (workspace Rust + CI) |
| **Última completada** | — |
| **Progreso** | Documentación base completa (vision, phases, adr, scaffolding) |

## Próximo paso

Fase 0 — Feature 0.1: escribir test RED de workspace (ej. `tpt_core_compila_y_expone_lib`), crear `Cargo.toml` workspace con los 4 crates skeleton (GREEN).

## Decisiones consolidadas (D1–D20)

Ver `docs/vision.md` (tabla D1–D20) y `docs/adr/`. Las críticas: D2 (hexagonal), D6–D11 (modos/penalizaciones), D12–D13 (inactividad/blocklist), D17–D18 (retención/BTRFS), D19 (presets).

## Decisiones pendientes

- Fórmula concreta del **Focus Score** (D16 lo deja configurable; definir pesos en Fase 7).
- Exacta definición de "día trabajado" (umbral de actividad > 0 min, configurable) en Fase 8.
- Presets default: confirmar `25/5`, `50/10`, `52/17`, `90/15(20)` como lista inicial.

## Riesgos activos / Gotchas

- ⚠️ **BTRFS + SQLite**: sin `chattr +C` en el directorio de la DB hay write amplification por CoW. Aplicar antes de crear la DB.
- ⚠️ **Idle sin libwayland**: la integración `ext-idle-notify-v1` debe ir tras `IdleSource`; evaluar helper `hypridle` o lectura de eventos Hyprland con fallback polling.
- ⚠️ **Hyprland socket2**: ruta depende de `$HYPRLAND_INSTANCE_SIGNATURE`; manejar ausencia con degradación elegante.
- ⚠️ **Presupuesto <15 MB RAM**: no agregar tokio ni dependencias pesadas al daemon; revisar cada crate en `Cargo.toml`.
- ℹ️ **Rama única `main`**: la regla de PR por fase se documenta; aplicará cuando exista remoto/ramas.

## Entorno

- OS: Arch Linux (Wayland/Hyprland).
- Toolchain: Rust estable + `cargo` (workspace multi-crate).
- Build/test: `cargo build` / `cargo test`.
- Lint: `cargo clippy --all-targets -- -D warnings`.
- TDD estricto: tests ANTES de implementar.
- Convención: código en inglés, docs en español (bloques de código con `rust`).