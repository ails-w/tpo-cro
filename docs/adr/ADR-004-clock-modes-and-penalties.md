# ADR-004: Modos de Reloj y Penalizaciones

- **Estado**: Aceptado
- **Fecha**: 2026-09-10

## Contexto

La visión original mezclaba "sesiones flexibles" y "anti-abandono" sin definir modos claros. El usuario pidió dos modos (Pomodoro y Focus-contrato) más Flowtime, ajuste por minuto, reinicio de ciclos, cooldown, challenge y reflexión. La evidencia (Smits et al. 2025; Ariely & Wertenbroch 2002; Gollwitzer & Sheeran 2006) muestra que no hay un método universal y que los *commitment devices* con pérdida (loss aversion) y fricción funcionan mejor que un timer informativo.

## Decisión

Tres modos (D6):

1. **Pomodoro** — estructurado, arrancador:
   - Sin pausa (D8). Abortar = reinicio de ciclos + sin racha de focus + reflexión obligatoria + challenge (D11).
   - `+`/`-` por minuto en trabajo y descansos, con step y límites configurables (D7).
   - Se completa solo si el tiempo **productivo** ≥ target (idle nunca lo completa solo).
2. **Focus (contrato duro)** — inmutable:
   - Sin `+`/`-`, sin pausa, sin stop (D7/D8).
   - Abortar = crédito 0 + cooldown 5 min visible en TUI (D9/D10) + challenge + reflexión obligatoria + impacto en métricas (`ABORTED_PENALIZED`).
3. **Flowtime** — medir, no imponer:
   - Sesión variable; descanso proporcional; sin penalización.

Reglas transversales:
- Challenge/contraseña + reflexión obligatoria en ambos modos para abortar (D11).
- Switch de tareas programadas a mitad de sesión en Pomodoro y Focus, sin penalización (D14).
- Presets editables `25/5`, `50/10`, `52/17`, `90/15(20)`; descanso corto ≤15, largo ≤20 (D19).

## Alternativas consideradas

- **Un solo modo híbrido (visión original)**: ambiguo; no distingue estructura vs compromiso.
- **Pomodoro con pausa**: contradice el objetivo de compromiso; el usuario lo descartó explícitamente.
- **Penalización monetaria/social**: fuera de scope (privacidad local).

## Consecuencias

- `focus_sessions` gana `mode`, `productive_duration_seconds`, `reflection`, `aborted_with` (ver `ADR-002`).
- El motor de relojes (Fase 5) es una state machine pura y testeable.
- Las métricas distinguen **observado vs acreditado** (D15): abortos Focus no acreditan.

## Referencias

- `docs/vision.md` (2.3) · `docs/phases.md` (Fase 5) · `ADR-002-sqlite-schema.md`