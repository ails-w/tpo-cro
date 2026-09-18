# ADR-003: Arquitectura Hexagonal — Puertos y Adaptadores

- **Estado**: Aceptado
- **Fecha**: 2026-09-14 · *(reestructurado por el pivote timer-first, `ADR-007`)*

## Contexto

El dominio (contrato de tiempo, penalizaciones, métricas) no debe depender del SO ni de Hyprland. Además, con el pivote cambió qué señales externas existen: ya no hay ventanas ni `/proc`, pero sí hay **presencia**.

## Decisión

Arquitectura hexagonal (D2): puertos (traits) en `tpt-core`, adaptadores en `tpt-daemon` y en los clientes.

### Puertos vigentes

| Puerto | Responsabilidad | Adaptador |
|--------|-----------------|-----------|
| `Clock` | Tiempo monotónico + wall-clock; detecta saltos por suspensión | `SystemClock` |
| `PresenceSource` | Eventos de presencia: `Idle{seconds}` · `Active` · `ScreenOff` · `Suspended{seconds}` · `Resumed` | `QuickshellHookSource` + `ClockGapSource` |
| `Store` | Actividades, entradas, sesiones, gaps, notas, contrato | `SqliteStore` |
| `ConfigSource` | Carga y validación de configuración | `TomlConfig` |
| `Notifier` | Avisos (escalada, fin de bloque, cooldown) | `NotifySender` |
| `IpcTransport` | Transporte UDS compartido | `UnixSocketTransport` |

### Puertos retirados

| Puerto | Motivo |
|--------|--------|
| `WindowSource` | El tracking de ventanas se abandonó (`ADR-007`) |
| `IdleSource` | Reemplazado por `PresenceSource`, que unifica idle + pantalla apagada + suspensión |

### Reglas de diseño

- **Los traits son pequeños y object-safe** (se usan como `Box<dyn _>` desde el daemon).
- **Todo adaptador tiene su fake** en `tpt-core` o en `tests/` para poder testear el dominio sin Hyprland, sin Wayland y sin disco.
- **Errores tipados con `thiserror`** en el dominio; nunca `unwrap`/`expect`/`panic` en `src/` (ver `AGENTS.md`).

## Alternativas consideradas

- **Un solo trait genérico de "fuente de eventos"**: evita proliferación, pero mezcla responsabilidades y hace los tests más frágiles. Descartada.
- **Mantener `IdleSource` y agregar `LockSource`/`PowerSource` por separado**: tres traits para una sola pregunta ("¿el usuario está presente?"). `PresenceSource` unifica.

## Consecuencias

- El core se testea en CI sin compositor ni sesión gráfica.
- Cambiar de compositor o de gestor de idle = nuevo adaptador, sin tocar el dominio.
- El daemon no arrastra dependencias de TUI/CLI.
- Costo: definir los traits antes de implementar (Fase 1).

## Referencias

- `docs/architecture.md` (Puertos) · `ADR-001` · `ADR-008` · `ADR-007`
