# Handoff — Continuidad

> ⚠️ **ESTADO MUTABLE.** Se SOBREESCRIBE al cerrar sesión. No es historial.
> Historial por fase → `docs/progress-log/`. Conceptos → `docs/learning/`.
> Aprendizajes y decisiones persistentes → Engram (memoria).

**Última actualización**: 2026-09-14

## Estado actual

| | |
|---|---|
| **Fase activa** | Fase 1 — Pivote documental + dominio, config y puertos |
| **Última completada** | Fase 0 — Setup (workspace Rust + CI) |
| **Progreso** | Feature 1.0 completa: pivote documental **y contrato cerrado** (`ADR-004` con niveles, deuda de reparación y refinanciación). Features 1.1–1.5 (dominio/config/puertos) pendientes. |

## Pivote de alcance (2026-09)

La app dejó de ser un **monitor pasivo de ventanas** y pasó a ser un **gestor de actividades con contrato de tiempo** (`ADR-007`).

- **Se retiró:** tracking de ventanas (Hyprland `socket2`), `$PWD` vía `/proc`, motor de reglas regex, categorías, blocklist de apps, tiers de retención.
- **Se conservó:** modos de reloj, métricas, rutinas, import de Super Productivity, arquitectura de 4 crates.
- **Se agregó:** presencia (`PresenceSource`), contrato aditivo, niveles de estrictez, tiempo manual etiquetado, reflexión opcional, agenda/cola de actividades.

## Próximo paso

Fase 1 — **Feature 1.1**: modelos de configuración. Test RED `app_config_parse_valid_toml_returns_expected`, luego modelos + parsing TOML con defaults (GREEN) en `crates/tpt-core/src/config/`.

## Decisiones consolidadas (D1–D20)

Ver `docs/vision.md` (tabla D1–D20) y `docs/adr/`. Las críticas para el pivote: D3 (presencia sin dependencias), D6–D9 (modos y contrato aditivo), D12 (inactividad amable), D16 (niveles + compromiso), D17 (agenda), D19 (SQLite sin tiers).

## Decisiones pendientes

- Fórmula concreta del **Focus Quality** (promedio de rating normalizado por `rating_scale`) → Fase 8.
- Umbral exacto de **"día trabajado"** (minutos mínimos) → Fase 8.
- Definición fina de la **racha**: día cumplido con ≥1 ciclo o ≥X min acreditados → Fase 9.
- ~~Niveles de estrictez y default~~ ✅ cerrado en `ADR-004` §5 (`L0` default, L1/L2 presets fijos).
- ~~Modelo de `Task`~~ ✅ resuelto: `Activity` es la unidad; el `Schedule` es opcional.
- ~~Challenge y cooldown~~ ✅ rediseñados como **deuda de reparación + refinanciación** (`ADR-004` §6–7).

## Riesgos activos / Gotchas

- ⚠️ **BTRFS + SQLite**: sin `chattr +C` en el directorio de la DB hay write amplification por CoW. Aplicar **antes** de crear la DB.
- ⚠️ **El compromiso es autoimpuesto**: sin tracking de apps, el escape real (navegador, celular) no está cubierto. Es una decisión consciente (`ADR-007`), no un pendiente.
- ⚠️ **`hypridle` debe estar corriendo** para avisos de idle y detección de lock. Sin él, solo queda el backstop de salto de reloj (degradación elegante, documentada en `ADR-008`).
- ⚠️ **Bajo consumo de memoria**: no agregar tokio ni dependencias pesadas al daemon. Por eso se descartó D-Bus (`ADR-008`).
- ℹ️ **Scope `workflow` de GitHub**: el token OAuth de `gh` no puede pushear ni mergear PRs que toquen `.github/workflows/`. Usar SSH, o un PAT con permiso Workflows (`ADR-006`).

## Entorno

- OS: Arch Linux (Wayland/Hyprland) con `hypridle`.
- Toolchain: Rust **1.98.1** del paquete de Arch (**sin `rustup`**); `rustfmt` y `clippy` incluidos.
- Edition del proyecto: **2024** (resolver 3), MSRV **1.85**.
- Build/test: `cargo build` / `cargo test`. Lint: `cargo clippy --all-targets -- -D warnings`.
- TDD estricto: tests ANTES de implementar.
- Convención: código en inglés, docs en español.
- Editor: Neovim (LazyVim) con `rust-analyzer` (Mason) + extra `lang.rust`.

## Git

- Remoto: `origin` → `github.com:ails-w/tpo-cro` (HTTPS en la config local; la llave SSH está registrada y tiene passphrase).
- Flujo: `feat/* → dev → main` (`ADR-006`).
- ⚠️ **Estado pendiente de Fase 0**: `dev` quedó creada desde `main` y **no contiene los commits de Fase 0**, que viven en la rama remota `chore/phase-0-setup` (`c6f36c9`). Falta llevarlos a `dev`.
