# ADR-004: Modos de Reloj, Contrato y Penalizaciones

- **Estado**: Aceptado
- **Fecha**: 2026-09-14 · *(reestructurado por el pivote timer-first, `ADR-007`)*

## Contexto

El diseño anterior definía "tiempo productivo" por categorización de ventanas. Sin tracking (`ADR-007`), esa definición desaparece. Hay que redefinir **qué es tiempo acreditado**, qué se puede hacer durante una sesión y cuánto cuesta salirse. El objetivo es un compromiso real, sin caer en castigos arbitrarios.

## Decisión

### 1. Invariantes del contrato

1. **No existe sesión sin actividad.** Siempre hay una `Activity` asociada.
2. **No existe pausa** en Pomodoro ni en Focus. **Flowtime sí tiene pausa** (es el modo libre).
3. **El contrato solo crece.** Se puede `+N` (extender); **nunca** `-N`. No se puede acortar para zafar.
4. **Cerrar la TUI no aborta.** La sesión vive en el daemon.
5. **El descanso de Pomodoro no acredita** a la actividad.
6. **El switch de actividad no penaliza:** el tiempo se parte y el contrato continúa (D14).

### 2. Modos

| Modo | Contrato | Durante la sesión | Salidas |
|------|----------|-------------------|---------|
| **Flowtime** | Libre, sin target | pausa / reanudar | Parar (sin penalización) |
| **Pomodoro** | Ciclos trabajo ↔ descanso | `+` para extender trabajo | Saltear descanso · Terminar sesión |
| **Focus** | Contrato duro e inmutable | solo `+` | Abortar (penalización según nivel) |

### 3. Presets (ajustables, D13)

`25/5` · `50/10` · `75/12` · `90/15`, con descanso largo `20`. Regla dura: **corto ≤ 15**, **largo ≤ 20**. `cycles_before_long` configurable por preset.

### 4. Tiempo acreditado vs observado

```
acreditado = transcurrido − Σ descuento(gap)

descuento(IDLE)      = max(0, gap − gracia)     # gracia default 180 s
descuento(LOCKED)    = gap                       # sin gracia
descuento(SUSPENDED) = gap                       # sin gracia
```

- La **gracia solo aplica a idle**: no tocar el teclado 2 min puede ser *pensar*. Pantalla bloqueada o máquina dormida no admiten interpretación.
- **El contrato se mide en tiempo acreditado**, no en reloj de pared. Dejar el reloj corriendo no alcanza: hay que estar.
- Los **avisos no descuentan tiempo**; solo informan.

### 5. Niveles de estrictez

| Nivel | Nombre | Detección de presencia | Avisos | Gap excedente | Penalización de tiempo |
|-------|--------|------------------------|--------|---------------|------------------------|
| `Off` | Sin presencia | ❌ | ❌ | se acredita todo | ninguna |
| `L0` | **Amable** (default propuesto) | ✅ | 3 y 5 min | **no se acredita**; la sesión sigue | ninguna |
| `L1` | Estricto | ✅ | 3, 5 y 7 min | no se acredita | si el no-acreditado ≥ `severe_ratio` (40%) del target → `ABORTED` + reflexión obligatoria |
| `L2` | Duro | ✅ | 3, 5 y 7 min | no se acredita | casos extremos → crédito 0 + cooldown 5 min + challenge + reflexión (`ABORTED_PENALIZED`) |

**Nivel 2 dispara solo si se cumple (A ∨ B ∨ C) ∧ D:**

- **A** un gap continuo ≥ 20 min, **o**
- **B** no-acreditado acumulado ≥ 40% del target, **o**
- **C** 3 avisos máximos alcanzados sin recuperación en la misma sesión, **y**
- **D** ya hubo ≥ 1 aborto por idle/lock en las últimas 24 h (strike previo).

Sin `D`, la primera vez cae en `L1`. **Nunca** dispara con `aborted_with = USER` ni en Flowtime.

> ⚠️ **Por discutir:** el *default* exacto y los umbrales de cada nivel. Los valores de la tabla son la propuesta.

### 6. Contrato de compromiso

Para que bajar la estrictez no sea un capricho de un día malo:

```
commitment_contracts { level, term_seconds, started_at, ends_at, checksum }
```

- **Subir** de nivel: inmediato, pero exige elegir un **término** (días / semanas / meses; mínimo 1 día).
- **Bajar** de nivel: **bloqueado** hasta `ends_at`.
- Vive en la **DB con checksum**, no en `config.toml` (que es editable a mano).
- Al vencer: renovar, subir o bajar libremente.

### 7. Reflexión

- Al **abortar**: obligatoria (challenge + reflexión).
- Al **completar**: opcional, pero se ofrece.
- **Rating 1–10 y notas son independientes y opcionales.** Una sesión puede tener solo nota, solo puntaje, ambos o nada.
- Se navegan en la vista `Historial` agrupada por fecha. Retención configurable (`notes_retention_days`, 0 = nunca).

## Alternativas consideradas

- **Pomodoro con pausa**: contradice el compromiso. Descartada.
- **Timer administrado por la TUI**: hacer `q` sería la vía de escape. Descartada → `ADR-001`.
- **Penalizar toda inactividad**: castiga leer/pensar y ensucia la métrica. Descartada → días amables (L0).
- **Nivel de estrictez en `config.toml`**: se edita con cualquier editor y el contrato se vuelve decorativo. Descartada.

## Consecuencias

- El motor de sesiones es una **máquina de estados pura y testeable** (Fase 4).
- Las métricas distinguen **observado vs acreditado** (D15) y por `source` (D18).
- Los abortos `ABORTED_PENALIZED` no acreditan.
- La detección de presencia es un puerto aparte → `ADR-008`.

## Referencias

- `docs/vision.md` (3.3–3.6) · `ADR-002` · `ADR-008` · `docs/phases.md` (Fase 4)
