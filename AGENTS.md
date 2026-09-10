# AGENTS.md - Guía de Ingeniería y Desarrollo Asistido por IA

Bienvenido al repositorio de **Terminal Productivity Tracker (TPT)**. Como agente de IA en OpenCode, debes actuar con el rigor de un Ingeniero de Software Senior especialista en Rust, sistemas Linux (Wayland/Hyprland) y TDD.

> Lo mutable (fase activa, próximo paso, riesgos) vive SOLO en `docs/handoff.md`.
> La documentación se navega desde `docs/index.md`.

---

## 📐 Principios de Arquitectura e Identidad

1. **Eficiencia Extrema:** El daemon no debe superar los 15MB de RAM bajo ninguna circunstancia. Evalúa el costo de memoria de cada crate antes de sugerir su adición a `Cargo.toml`.
2. **Cero Tolerancia a Panics:** Se prohíbe el uso de `.unwrap()`, `.expect()` o `panic!()` en código de producción (`src/`). Todo error debe propagarse usando `thiserror` (en librerías/core) o manejarse con resiliencia en la app.
3. **Desacoplamiento del Sistema Operativo:** Todo acceso a sockets de Wayland, ejecuciones CLI (`hyprctl`) o lectura de `/proc` DEBE estar aislado detrás de un `Trait` (Inversion of Control). Arquitectura hexagonal: puertos en `tpt-core`, adaptadores en `tpt-daemon` (ver `docs/adr/ADR-003-hexagonal-ports.md`).

---

## 🧪 Metodología TDD Estricta (Red - Green - Refactor)

No escribas código de implementación sin antes haber creado y fallado el test correspondiente.

1. **Paso 1 (RED):** Escribe el test unitario o de integración en `tests/` o en un módulo `#[cfg(test)]`. Ejecuta `cargo test` y confirma que falla por las razones correctas.
2. **Paso 2 (GREEN):** Escribe el código MÍNIMO necesario para hacer pasar el test.
3. **Paso 3 (REFACTOR):** Limpia el código, optimiza asignaciones de memoria (evita clones innecesarios de `String`), y verifica que `cargo clippy --all-targets -- -D warnings` pase totalmente limpio.

---

## 📦 Estructura del Workspace Rust

```text
.
├── Cargo.toml               # Workspace manifest
├── crates/
│   ├── tpt-core/            # Modelos, puertos (traits), reglas, categorización, importers
│   ├── tpt-daemon/          # Event loop, adaptadores Hyprland, idle, UDS Server, Store
│   ├── tpt-tui/             # Interfaz Ratatui, UDS Client, Event loop de teclado
│   └── tpt-cli/             # Cliente de control/report/import por comandos (D1)
├── docs/                    # Documentación (index, vision, phases, adr, learning, progress-log)
├── config.toml              # Configuración de ejemplo/por defecto
└── .github/workflows/       # CI/CD Pipeline
```

---

## 📚 Reglas de Documentación

- **Idioma:** código en inglés, docs en español. Nombres de carpetas/archivos en inglés.
- **`docs/phases.md` es POR FEATURE:** cada fase entrega una feature concreta con scope, criterios de salida y pasos TDD.
- **`docs/learning/` y `docs/progress-log/` se desarrollan MIENTRAS se avanza el proyecto** (justo-a-tiempo, no al final). Cada fase crea `learning/phase-NN-name.md` y `progress-log/phase-NN-name.md` al comenzar.
- **`docs/handoff.md` es estado mutable:** actualizarlo al iniciar/cerrar sesión. No es historial.
- **ADRs:** una decisión = un archivo en `docs/adr/`; nunca reescribir historial (superceder con ADR nuevo).
- Formato: Markdown. Diagramas ASCII/Mermaid solo si aportan (`docs/diagrams/` opcional).

---

## 🗂️ Git + Definition of Done

- **Commits convencionales en inglés**: `feat:`, `fix:`, `test:`, `docs:`, `refactor:`, `chore:`.
- **REGLA — commit después de cada FEATURE:** al cerrar cada feature (test RED → GREEN → refactor), hacer un commit convencional de esa feature. No acumular.
- **REGLA — PR después de cada FASE terminada:** al completar una fase (criterios de salida cumplidos + log + learning + handoff actualizados), abrir una PR de revisión. Hoy solo existe la rama `main`; cuando haya remoto/más ramas (ej. `dev`), la PR se crea contra la rama correspondiente y el proceso se documenta en `docs/development-plan.md`.
- **NUNCA** añadir "Co-Authored-By" ni atribución IA.
- DoD de una feature: test RED que pasa (GREEN) + refactor + aprendizaje documentado en `docs/learning/phase-NN-name.md` + `docs/handoff.md` actualizado + commit convencional.

---

## 🔧 Comandos

- Build: `cargo build`
- Test: `cargo test`
- Lint: `cargo clippy --all-targets -- -D warnings`
- Format: `cargo fmt --check`
- Test individual: `cargo test <nombre_del_test>`
- Run daemon: `cargo run -p tpt-daemon`
- Run TUI: `cargo run -p tpt-tui`
- Run CLI: `cargo run -p tpt-cli -- <comando>`

---

## 🧪 Testing

- Framework: `cargo test` (unit en `#[cfg(test)]` + integración en `tests/`).
- Pirámide: unit (core, rápido, aislado) → integración (adaptadores con fakes + SQLite temporal) → funcional (daemon real, opcional en CI).
- Nombre de tests: `fn <método>_<condición>_<esperado>()`.
- Los tests son parte de la definición de "terminado" (DoD).

---

## 🚫 Límites / Do-nots

- NO tocar `target/` a mano.
- NO escribir implementación sin test previo (TDD estricto).
- NO duplicar contenido entre archivos; si hay que copiar un párrafo, está mal ubicado.
- NO modificar `docs/handoff.md` salvo al iniciar/cerrar sesión.
- NO usar `.unwrap()`, `.expect()` o `panic!()` en `src/` de producción.

---

## 📍 Tabla de punteros

| Área | Documento |
|---|---|
| **Estado actual** (LEER al iniciar, ACTUALIZAR al cerrar) | `docs/handoff.md` |
| Mapa de navegación completo | `docs/index.md` |
| Plan de fases POR FEATURE (scope + criterios de salida) | `docs/phases.md` |
| Visión y especificación funcional | `docs/vision.md` |
| Arquitectura y esquema | `docs/architecture.md` |
| Decisiones de arquitectura | `docs/adr/` |