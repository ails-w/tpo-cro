# ADR-001: Arquitectura General, Persistencia e IPC

- **Estado**: Aceptado
- **Fecha**: 2026-09-10

## Contexto

El sistema requiere monitorear continuamente la actividad de ventanas, rutas ($PWD) e inactividad en Arch Linux (Hyprland), consumiendo menos de 15 MB de RAM. La interfaz TUI debe poder abrirse y cerrarse sin interrumpir el daemon de fondo.

## Decisión

1. **Separación de Procesos:**
   - `tpt-daemon`: Proceso en segundo plano sin UI. Encargado del event loop de captura, idle detection, categorización y persistencia.
   - `tpt-tui`: Cliente visual ejecutable a demanda usando `ratatui` + `crossterm`.
   - `tpt-cli`: Cliente de control/report/import por comandos (ver `ADR-007-tpt-cli-crate.md`).

2. **IPC (Unix Domain Sockets):**
   - Protocolo: UDS (`$XDG_RUNTIME_DIR/tpt.sock`).
   - Formato de Mensaje: Framing con longitud delimitada en JSON (Serializado vía `serde_json`).
   - Bidireccional: El daemon notifica cambios de estado a la TUI; la TUI envía comandos de control de Pomodoro al daemon.
   - Protocolo versionado (`protocol_version` + `min_supported`) y socket con permisos `0600` (D4).

3. **Persistencia (SQLite síncrono):**
   - Crate: `rusqlite` con modo `WAL` (Write-Ahead Logging) activado para lecturas concurrentes rápidas.
   - Hilo dedicado para I/O de base de datos usando canales MPSC (`std::sync::mpsc`) para asegurar que el registro de eventos no bloquee la captura en tiempo real.
   - Retención modular por tiers y estrategia BTRFS (ver `ADR-006-data-retention-btrfs.md`).

4. **Integración con Hyprland:**
   - Event-driven mediante lectura del socket IPC de Hyprland (`$XDG_RUNTIME_DIR/hypr/$HYPRLAND_INSTANCE_SIGNATURE/.socket2.sock`) combinado con `ext-idle-notify-v1`, todo tras traits (D2/D3).

5. **Arquitectura hexagonal (D2):**
   - Puertos (traits) en `tpt-core`: `WindowSource`, `IdleSource`, `Clock`, `Store`, `ConfigSource`, `Notifier`, `IpcTransport`.
   - Adaptadores en `tpt-daemon` (Hyprland, SQLite, TOML, system clock).

## Alternativas consideradas

- **Runtime async (tokio)**: overhead de memoria incompatible con el objetivo <15 MB.
- **IPC por HTTP/WebSocket**: superficie de autenticación innecesaria para un canal local.

## Consecuencias

- Cero overhead de runtimes async pesados en el proceso crítico.
- Consumo de RAM estimado: Daemon (~4-8 MB), TUI (~6-10 MB).
- Facilidad de testeo en entornos CI/CD sin servidor X11/Wayland activo (adaptadores fake).

## Referencias

- `docs/vision.md` · `docs/architecture.md` · `ADR-002-sqlite-schema.md` · `ADR-003-hexagonal-ports.md`