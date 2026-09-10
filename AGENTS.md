# AGENTS.md - Guía de Ingeniería y Desarrollo Asistido por IA

Bienvenido al repositorio de **Terminal Productivity Tracker (TPT)**. Como agente de IA en OpenCode, debes actuar con el rigor de un Ingeniero de Software Senior especialista en Rust, sistemas Linux (Wayland/Hyprland) y TDD.

---

## 📐 Principios de Arquitectura e Identidad

1. **Eficiencia Extrema:** El daemon no debe superar los 15MB de RAM bajo ninguna circunstancia. Evalúa el costo de memoria de cada crate antes de sugerir su adición a `Cargo.toml`.
2. **Cero Tolerancia a Panics:** Se prohíbe el uso de `.unwrap()`, `.expect()` o `panic!()` en código de producción (`src/`). Todo error debe propagarse usando `thiserror` (en librerías/core) o manejarse con resiliencia en la app.
3. **Desacoplamiento del Sistema Operativo:** Todo acceso a sockets de Wayland, ejecuciones CLI (`hyprctl`) o lectura de `/proc` DEBE estar aislado detrás de un `Trait` (Inversion of Control).

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
│   ├── tpt-core/            # Modelos, DB (rusqlite), Categorización Regex, Importers
│   ├── tpt-daemon/          # Event loop, Wayland IPC listener, Idle detector, UDS Server
│   └── tpt-tui/             # Interfaz Ratatui, UDS Client, Event loop de teclado
├── docs/                    # ADRs y especificaciones técnicas
└── .github/workflows/       # CI/CD Pipeline
