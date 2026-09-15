# ADR-008: Modelo de Tracking en Tres Capas (Categorización, Asignación, Blocklist)

- **Estado**: Aceptado
- **Fecha**: 2026-09-14

## Contexto

Al discutir "qué trackea el timer" aparece una ambigüedad recurrente: se asume que para registrar tiempo hace falta declarar, por tarea, una lista de apps (permitidas o prohibidas). No es así. `ADR-005` definió la blocklist por tarea, pero nunca quedó explícito que el **tracking pasivo** y el **enforcement** son responsabilidades separadas, ni que la **categorización es global** (no por tarea). Sin esa separación, el diseño del motor de reglas, del esquema y del motor de relojes queda ambiguo.

## Decisión

El tracking se modela en **tres capas independientes**. Ninguna exige una "lista de apps permitidas por tarea".

### Capa 1 — Categorización (tracking pasivo, global)

- Responde: *¿esta actividad es productiva o distracción?*
- Vive en `config.toml`: `[[categories]]` (`is_productive`) + `[[rules]]` (regex sobre `class`/`title`, y `pwd_prefix`).
- Se aplica a **toda** ventana activa, sin importar la tarea.
- El daemon **registra todo** lo observado en `window_activity_logs`; la categoría solo etiqueta. Sin regla que aplique, el registro queda sin categoría (no se descarta).

### Capa 2 — Asignación a tarea/proyecto

- Responde: *¿a qué tarea o proyecto se acredita este tiempo?*
- Fuentes: `$PWD` del terminal (`/proc/<pid>/cwd`, canonicalizado) o la tarea activa de la sesión.
- Una ruta de trabajo configurada gana sobre las reglas genéricas cuando coincide con un proyecto.

### Capa 3 — Blocklist (enforcement, por tarea)

- Responde: *¿qué NO puedo abrir durante este bloque?*
- Vive en `task_blocked_apps` (por tarea). Es una lista de apps **prohibidas**, nunca una whitelist (ver `ADR-005`).
- Solo aplica **durante una sesión activa** (Pomodoro/Focus). Fuera de sesión no hay enforcement.
- Cambiar de ventana entre apps no bloqueadas es libre: nunca se penaliza.

### Cómo cuenta el timer

El motor de relojes **no conoce apps**. Distingue:

- **Tiempo observado** (`actual_duration_seconds`): todo lo registrado durante la sesión.
- **Tiempo productivo** (`productive_duration_seconds`): lo que cae en categorías con `is_productive = true` (o lo asignado al proyecto vía Capa 2), **descontando idle**.

Una sesión se completa sobre el tiempo **productivo**; el idle nunca la completa solo (ver `ADR-004`).

## Alternativas consideradas

- **Whitelist de apps por tarea**: obliga a enumerar todo lo permitido y castiga contextos no previstos; menos descriptiva (descartada en `ADR-005`).
- **Lista de apps para trackear**: innecesaria; el tracking es pasivo y total, la categorización la resuelven reglas globales.
- **Blocklist global (no por tarea)**: pierde granularidad; la misma app puede ser distracción en una tarea y herramienta en otra.

## Consecuencias

- El motor de reglas (Fase 3) es global y testeable sin conocer tareas ni sesiones.
- La blocklist (Fase 4) es enforcement puro y depende de la tarea activa, no del tracking.
- El motor de relojes (Fase 5) consume categorías y tiempo productivo; no maneja listas de apps.
- Agregar una categoría o una regla no toca tareas ni sesiones.
- El modelo concreto de `Task` (¿solo rutinas recurrentes de `ADR-002`, o también tareas ad-hoc con su propia blocklist?) se define en Fase 1; la blocklist cuelga de ese modelo.

## Referencias

- `docs/vision.md` (2.1, 2.3, 2.7) · `docs/phases.md` (Fases 3, 4, 5)
- `ADR-002-sqlite-schema.md` (`window_activity_logs`, `task_blocked_apps`)
- `ADR-005-inactivity-and-blocklist.md` (blocklist vs whitelist)
