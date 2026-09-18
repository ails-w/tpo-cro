# ADR-001: Arquitectura General, Persistencia e IPC

- **Estado**: Aceptado
- **Fecha**: 2026-09-14 · *(reestructurado por el pivote timer-first, `ADR-007`)*

## Contexto

La app gestiona **contratos de tiempo** sobre actividades. El compromiso es el producto: si la sesión viviera en el proceso de la TUI, cerrarla con `q` sería la vía de escape trivial. Además se exige consumo de memoria mínimo (sin runtime async) y persistencia local confiable.

## Decisión

1. **Separación de procesos**
   - `tpt-daemon`: proceso de fondo sin UI. Es **dueño de la sesión** (estado, contrato, penalizaciones), del reloj, de la presencia y de la persistencia.
   - `tpt-tui`: cliente a demanda (`ratatui` + `crossterm`). Abrirla o cerrarla **no interrumpe ni aborta** una sesión.
   - `tpt-cli`: cliente de control, reporte, import y ciclo de vida del daemon (ver `ADR-005`).

2. **IPC (Unix Domain Sockets)**
   - Socket en `$XDG_RUNTIME_DIR/tpt.sock`, framing longitud + JSON (`serde_json`), permisos `0600`.
   - Bidireccional: la TUI manda comandos; el daemon emite eventos (escalada de presencia, fin de bloque).
   - Protocolo versionado (`protocol_version` + `min_supported`).

3. **Persistencia (SQLite síncrono)**
   - `rusqlite` con `WAL` para lecturas concurrentes.
   - Un **hilo escritor dedicado** con canal MPSC (`std::sync::mpsc`) para no bloquear el loop de sesión.
   - Esquema y estrategia de agregación → `ADR-002`.

4. **Presencia (idle / pantalla apagada / power)**
   - Fuente primaria: hooks de `caelestia-shell` (quickshell). Backstop: salto de reloj monotónico vs wall-clock.
   - Detalle y argumentación → `ADR-008`.

5. **Arquitectura hexagonal**
   - Puertos (traits) en `tpt-core`; adaptadores en `tpt-daemon`.
   - Detalle → `ADR-003`.

## Alternativas consideradas

- **TUI monolítica (sin daemon)**: más simple, pero hacer `q` terminaría la sesión → **el compromiso sería decorativo**. Descartada.
- **Runtime async (tokio)**: overhead de memoria incompatible con el objetivo de bajo consumo. Descartada.
- **IPC por HTTP/WebSocket**: superficie de autenticación innecesaria para un canal local. Descartada.
- **Tracking pasivo de ventanas** (diseño original): abandonado en `ADR-007`.

## Consecuencias

- Cerrar la TUI **no** es una vía de escape: la sesión sigue viva en el daemon.
- El core se testea en CI sin Hyprland ni Wayland, usando fakes de los puertos.
- Cero overhead de runtimes async en el proceso crítico.
- El consumo de memoria se mide y documenta en Fase 10; no se declaran límites de antemano.

## Referencias

- `docs/vision.md` · `docs/architecture.md` · `ADR-002` · `ADR-003` · `ADR-008`
