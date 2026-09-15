# Fase 00 — Setup del workspace

> Conceptos de la fase 0 del proyecto (infraestructura del workspace Rust y CI).
> Log de la fase → `docs/progress-log/phase-00-setup.md`

---

## Workspace multi-crate en Rust

### Qué es

Un **workspace** es un conjunto de crates que comparten una raíz común: un único `Cargo.lock`, un único directorio `target/`, y configuración heredable (`[workspace.package]`, `[workspace.lints]`, `[workspace.dependencies]`). El `Cargo.toml` de la raíz es un **virtual manifest**: tiene `[workspace]` pero **no** tiene `[package]`.

### Qué problema resuelve

Sin workspace, cada crate sería un proyecto independiente: 4 `Cargo.lock`, 4 `target/` (builds repetidos), versiones de dependencias que pueden divergir, y ningún lugar único donde fijar lints o edition. Con workspace, una sola resolución de dependencias y configuración compartida.

### Para qué sirve en este proyecto

TPT se parte en 4 crates con dependencias muy distintas (`tpt-core` es dominio puro; `tpt-daemon` toca el SO; `tpt-tui` arrastra ratatui; `tpt-cli` arrastra clap). El workspace permite que el daemon **no** arrastre dependencias de TUI/CLI — requisito de "Eficiencia Extrema" de `AGENTS.md` — mientras todo se compila, testea y lintea de una sola vez.

### Cómo se usa

Raíz (`Cargo.toml`):

```toml
[workspace]
resolver = "3"
members = ["crates/tpt-core", "crates/tpt-daemon", "crates/tpt-tui", "crates/tpt-cli"]
```

Un crate hereda lo común con `.workspace = true`:

```toml
[package]
name = "tpt-core"
version.workspace = true
edition.workspace = true
license.workspace = true

[lints]
workspace = true
```

### Error común

- Poner `[package]` en el `Cargo.toml` raíz junto a `[workspace]` (no puede ser virtual y real a la vez).
- Creer que cada crate necesita su propio `Cargo.lock`; en un workspace **solo existe uno**, en la raíz.
- Olvidar `[lints] workspace = true` en el crate: los lints del workspace **no** se aplican solos.

### Referencias

- Cargo Book — *Workspaces*: https://doc.rust-lang.org/cargo/reference/workspaces.html
- `docs/architecture.md` (estructura de carpetas)

---

## `cargo clippy --all-targets -- -D warnings` como puerta

### Qué es

`clippy` es el linter de Rust. `--all-targets` lo corre sobre **todos** los targets (lib, bin, tests, benches, examples), no solo la lib. Y `-- -D warnings` convierte cada warning en **error**, lo que hace que el comando falle con exit code ≠ 0 si algo no está limpio.

### Qué problema resuelve

Un warning que no rompe el build se ignora para siempre y se acumula. Al promoverlo a error, la limpieza deja de ser opcional: el código no avanza con deuda de lint. Es la diferencia entre "recomendación" y "puerta".

### Para qué sirve en este proyecto

`AGENTS.md` exige que `cargo clippy --all-targets -- -D warnings` pase limpio en el REFACTOR de cada feature. Es el tercer paso del ciclo TDD del repo y la mitad de la DoD. Acá se combina con los lints de "cero panics" (ver el concepto siguiente).

### Cómo se usa

```bash
cargo clippy --all-targets -- -D warnings   # local
```

En CI se corre igual, pero acotado al workspace y contra el lock:

```bash
cargo clippy --workspace --all-targets --locked -- -D warnings
```

### Error común

- Correr `cargo clippy` a secas: solo lintea la lib del crate actual, y los tests pasan sin ser revisados.
- Olvidar el `--` antes de `-D warnings`: sin el separador, cargo interpreta los flags como propios y el comando falla o los ignora.

### Referencias

- Clippy lints index: https://doc.rust-lang.org/clippy/
- `AGENTS.md` (metodología TDD) · `docs/development-plan.md` (CI)

---

## Lints de crate y edition

### Qué es

- **Edition**: cada crate declara en qué edición de Rust se compila (`edition = "2024"`). Las ediciones son *opt-in* y a nivel de crate; conviven sin problema.
- **Resolver**: `resolver = "3"` es el algoritmo de resolución de features; es el que corresponde a edition 2024.
- **Lints de workspace**: `[workspace.lints]` centraliza la política de lints, y cada crate la adopta con `[lints] workspace = true`.

### Qué problema resuelve

Sin esto, cada crate elegiría su edición y su nivel de lints por separado, y la política se dispersa. Con `[workspace.lints]` hay **una** fuente de verdad para "qué consideramos código aceptable".

### Para qué sirve en este proyecto

`AGENTS.md` prohíbe `.unwrap()`, `.expect()` y `panic!()` en producción. Eso se traduce a lints de *restriction* en la raíz:

```toml
[workspace.lints.rust]
unsafe_code = "forbid"

[workspace.lints.clippy]
all = { level = "warn", priority = -1 }
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
```

`priority = -1` hace que el grupo `all` se aplique primero y los lints específicos puedan overridearlo. Verificado: meter un `unwrap()` temporal en `src/` rompe `cargo clippy`.

### Cómo se usa

Los lints `deny` aplican también a los tests (`--all-targets`), donde `unwrap()` es idiomático. Se exceptúan de forma acotada, no global:

```rust
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
```

Esto habilita la excepción **solo** cuando se compila el harness de tests, no en el binario de producción.

### Error común

- Denegar `unwrap_used` sin exceptuar los tests: rompe todas las aserciones que usan `.unwrap()`.
- Exceptuar con `allow` global en vez de `cfg_attr(test, ...)`: abre la puerta en producción, justo donde la querés cerrada.
- Usar `edition = "2024"` con `resolver = "2"`: mezcla ediciones y resolución; el default de 2024 es el resolver 3.

### Referencias

- Rust Edition Guide: https://doc.rust-lang.org/edition-guide/
- Lints config: https://doc.rust-lang.org/cargo/reference/manifest.html#the-lints-section

---

## Relación entre estos conceptos

Los tres forman la infraestructura mínima de la Fase 0: el **workspace** da la estructura, la **edition + lints** dan la política de código (una sola, centralizada), y **clippy como puerta** hace que esa política se cumpla de verdad. Sin workspace no hay dónde centralizar; sin centralizar, cada crate deriva; sin la puerta, la política es decorativa.

---

## Convención

- **Archivo**: `docs/learning/phase-00-setup.md`
- **Título**: `# Fase 00 — Setup del workspace`
- Cada fase crea su archivo al comenzar (un solo archivo por fase).
