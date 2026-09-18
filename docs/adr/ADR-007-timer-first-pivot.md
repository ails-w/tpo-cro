# ADR-007: Pivote a *Timer-First*

- **Estado**: Aceptado
- **Fecha**: 2026-09-14
- **Reemplaza**: el diseño con tracking pasivo de ventanas (ADR-005, ADR-006 y ADR-008 de la numeración anterior)

## Contexto

El diseño original era un **monitor pasivo**: capturaba la ventana activa por el socket IPC de Hyprland, leía `$PWD` de `/proc/<pid>/cwd`, categorizaba con reglas regex y aplicaba una blocklist de apps por tarea. La Fase 0 estaba cerrada y el código era solo scaffolding, cuando el usuario reconsideró el alcance:

> "Quiero dejar de lado el tracking de apps. Quiero enfocarme en timer, métricas, y que el reloj tenga sus restricciones fuertes cuando se inicia. Pero también quiero poder agregar tiempo manual a actividades creadas."

El objetivo pasó a ser: **un Super Productivity, pero con un motor de sesiones restrictivo propio**, y una app más simple.

## Decisión

**Se abandona el tracking pasivo. La app pasa a ser timer-first.**

### Se retira

| Elemento | Motivo |
|----------|--------|
| `WindowSource` (Hyprland `socket2`) | Sin tracking de apps |
| Detección de `$PWD` vía `/proc/<pid>/cwd` | Sin tracking de apps |
| Motor de reglas regex + `categories.is_productive` | La categorización era la base del tracking |
| Blocklist por tarea (`task_blocked_apps`) | Requiere conocer la app activa |
| `window_activity_logs` y los tiers de retención | Sin flood de eventos que retener |

### Se conserva y refuerza

- Motor de modos: **Flowtime / Pomodoro / Focus**, con contrato aditivo (`ADR-004`).
- **Actividades** creadas por el usuario, con agenda opcional y cola del día.
- **Tiempo manual** por actividad, etiquetado y sumado al total.
- **Métricas** como core del producto: heatmap, jerarquías año→día, promedios, Focus Quality.
- **Reflexión y notas** por sesión, opcionales e independientes.
- Import de Super Productivity.
- Daemon + TUI + CLI + IPC (`ADR-001`, `ADR-005`).

### Se agrega

- **`PresenceSource`** (`ADR-008`): idle, pantalla apagada y suspensión/apagado.
- **Niveles de estrictez** y **contrato de compromiso** (`ADR-004`).
- **Vista `Historial`** para notas y puntajes agrupados por fecha.

## Alternativas consideradas

- **Mantener tracking y sumarle el timer**: duplicaba complejidad (dos fuentes de verdad del tiempo) y obligaba a mantener el adaptador de Hyprland. Descartada.
- **Tracking opcional (feature flag)**: el costo de mantener el motor de reglas y el adaptador no se justificaba para un uso marginal. Descartada.
- **Sin daemon (todo en la TUI)**: el compromiso sería evadible con `q`. Descartada → `ADR-001`.

## Consecuencias

- **El compromiso pasa a ser autoimpuesto.** Ya no hay enforcement sobre apps: el escape real es el navegador o el teléfono, y la app no lo controla. Es una decisión consciente, no un olvido.
- Se elimina toda dependencia de Hyprland del dominio → el core es más portable y más fácil de testear.
- El esquema de datos se simplifica mucho (`ADR-002`): menos tablas, sin tiers, sin agregados precalculados.
- Las métricas ganan peso como funcionalidad principal, no como reporte secundario.

## Referencias

- `docs/vision.md` · `ADR-002` · `ADR-004` · `ADR-008` · `docs/phases.md`
