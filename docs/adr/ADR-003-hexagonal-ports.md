# ADR-003: Arquitectura Hexagonal — Puertos y Adaptadores

- **Estado**: Aceptado
- **Fecha**: 2026-09-10

## Contexto

TPT depende del OS (Hyprland IPC, `/proc`, idle, SQLite, reloj). Si el dominio conoce esas dependencias, no es testeable sin Wayland/X11 y queda acoplado a Hyprland. Además el presupuesto <15 MB RAM exige que el proceso crítico no arrastre dependencias de UI/reporte.

## Decisión

Arquitectura hexagonal (D2):

- **`tpt-core`** = dominio puro + **puertos** (traits), sin dependencias de SO:
  - `WindowSource` — ventana activa (class, title, pid, pwd).
  - `IdleSource` — señal de inactividad input.
  - `Clock` — monotónico + wall-clock.
  - `Store` — repositorio (logs, sesiones, agregados, retención).
  - `ConfigSource` — carga/validación de configuración.
  - `Notifier` — notificaciones del daemon.
  - `IpcTransport` — transporte UDS compartido.
- **`tpt-daemon`** = adaptadores: `HyprlandWindowSource`, `HyprlandIdleSource` (sin libwayland, D3), `SqliteStore`, `TomlConfig`, `SystemClock`.
- **`tpt-tui` / `tpt-cli`** = adaptadores *driving* (clientes IPC).
- Errores tipados con `thiserror` en el dominio.

## Alternativas consideradas

- **Dominio concreto acoplado a Hyprland**: rápido de escribir, pero no testeable fuera de la sesión del usuario y difícil de reemplazar.
- **Librería de abstracción genérica (p. ej. `wayland-client` genérico)**: liga memoria/deps al daemon sin necesidad.

## Consecuencias

- El core se testea en CI sin X11/Wayland usando fakes.
- Cambiar de Hyprland a otro compositor = nuevo adaptador, sin tocar el dominio.
- El daemon no arrastra dependencias de TUI/CLI (reporte/import) → RAM y build acotados.
- Costo: definir traits desde el inicio (Fase 1).

## Referencias

- `docs/architecture.md` (tabla de puertos) · `docs/vision.md` (D2) · `docs/phases.md` (Fase 1)