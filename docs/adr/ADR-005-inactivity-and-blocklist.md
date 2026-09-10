# ADR-005: Inactividad y Blocklist por Tarea

- **Estado**: Aceptado
- **Fecha**: 2026-09-10

## Contexto

El tracking pasivo no debe castigar al usuario por no tocar el teclado (leer, pensar, ver una demo). La visión original penalizaba inactividad sin matices. Además, el usuario pidió saber **qué apps NO se pueden abrir** durante un bloque, y que cambiar de ventana entre apps no bloqueadas nunca se penalice.

## Decisión

**Inactividad (D12): la falta de input es un dato, no una infracción.**
- Aviso informativo (no punitivo) tras `warn_after_seconds` (default 300 s): *"¿Seguís con la sesión? El tiempo idle no cuenta como productivo."*
- El gap idle se registra como `idle/offline` y se descuenta del acumulado productivo (no infla métricas, no rompe racha, no reinicia ciclos).
- Al volver, modal de clasificación (descanso vs actividad offline).
- Suspensión: el gap se trata como idle/offline (D5).

**Blocklist por tarea (D13):**
- Cada tarea/proyecto declara **apps prohibidas** (`task_blocked_apps`). Es más descriptivo que una whitelist: al iniciar el bloque sabés exactamente qué NO podés abrir.
- **Cambiar de ventana entre apps no bloqueadas es libre** — nunca se penaliza.
- Abrir una app bloqueada → aviso → countdown **3–5 min** (`violation_grace_seconds`, default 180 s) → si no la cerrás, penalización según modo:
  - **Pomodoro:** reinicio de ciclos (métricas intactas).
  - **Focus:** crédito 0 + cooldown 5 min + challenge + reflexión + registro en métricas.
- Máx **2 avisos por sesión** (`max_warnings_per_session`); al 3º, penalización directa.

## Alternativas consideradas

- **Whitelist (apps permitidas)**: obliga a enumerar todo lo permitido y castiga contextos no previstos; menos descriptivo para el usuario.
- **Penalizar por no-teclear con umbral**: castiga lectura/pensamiento y daña métricas; descartado.

## Consecuencias

- `IdleSource` es informativo; el enforcement real vive en el motor de blocklist.
- Esquema agrega `task_blocked_apps` (ver `ADR-002`).
- El flujo aviso → countdown → penalización es una máquina de estados testeable (Fase 4).

## Referencias

- `docs/vision.md` (2.7) · `docs/phases.md` (Fase 4) · `ADR-002-sqlite-schema.md`