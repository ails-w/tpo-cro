# ADR-004: Modos de Reloj, Contrato y Penalizaciones

- **Estado**: Aceptado
- **Fecha**: 2026-09-14 · *(reestructurado por el pivote timer-first, `ADR-007`)*

## Contexto

Sin tracking de apps (`ADR-007`), el "tiempo productivo" ya no puede salir de categorizar ventanas. Hay que redefinir qué se acredita, qué se puede hacer durante una sesión y cuánto cuesta romper el compromiso. El objetivo: un contrato real que **no castigue la honestidad** ni dependa de que el usuario se autodiscipline.

## Decisión

### 1. Invariantes

1. **No existe sesión sin actividad.**
2. **No existe pausa** en Pomodoro ni en Focus. **Flowtime sí tiene pausa.**
3. **El contrato solo crece.** Se puede `+N`; **nunca** `-N`.
4. **Cerrar la TUI no aborta.** La sesión vive en el daemon.
5. **El descanso no acredita** tiempo a la actividad.
6. **El switch de actividad no penaliza**: el tiempo se parte y el contrato continúa.
7. **El contrato pertenece a la sesión, no a la actividad.** Cualquier actividad, `timer` o `manual`, queda sujeta al contrato **mientras haya timer corriendo**.

### 2. Modos

| Modo | Contrato | Durante | Salidas |
|------|----------|---------|---------|
| **Flowtime** | Libre, sin target | pausa / reanudar | Parar (gratis) |
| **Pomodoro** | N ciclos trabajo ↔ descanso | `+` para extender trabajo | Ver §3 |
| **Focus** | Duro, target inmutable | solo `+` | Abortar (con costo) |

Presets ajustables: `25/5` · `50/10` · `75/12` · `90/15`, descanso largo `20`. Regla dura: **corto ≤15**, **largo ≤20**.

### 3. Pomodoro: salidas libres (por qué no es una trampa)

El Pomodoro **no es infinito**: tiene **N ciclos explícitos** (configurable por preset; default 4). Y salir es gratis en los bordes:

| Momento en que terminás | Resultado |
|-------------------------|-----------|
| Durante un **descanso** | `COMPLETED` — gratis |
| En el **borde de ciclo** (trabajo cumplido) | `COMPLETED` — gratis |
| Al completar los **N ciclos** | `COMPLETED` — gratis |
| **En medio de un ciclo de trabajo** | `ABORTED` — con deuda |

Además existe la acción **"Terminar al final del ciclo"**: la sesión se cierra sola al terminar el ciclo en curso. **Nunca hay que abortar para irse a hacer otra cosa.**

**Confirmación tras el descanso:** al terminar un descanso la sesión entra en `AWAITING` y **no avanza sola**. El siguiente ciclo de trabajo arranca **solo cuando el usuario confirma**. Si no vuelve dentro de `awaiting_expiry_minutes` (default 45), la sesión se cierra como `COMPLETED` con los ciclos ya cumplidos: **sin deuda y sin romper la racha.**

### 4. Tiempo acreditado vs observado

```
acreditado = transcurrido − Σ descuento(gap)

un gap CALIFICA si duración ≥ min_gap_seconds (default 7 min)
descuento("full")   = gap completo          (default)
descuento("excess") = max(0, gap − min_gap)
```

- Un gap por debajo del umbral **no existe**: no descuenta ni acumula.
- **Los avisos nunca descuentan tiempo.** El descuento lo produce la ausencia real.
- **El contrato se mide en tiempo acreditado**, no en reloj de pared.

### 5. Niveles de estrictez

| Parámetro | `Off` | `L0` | `L1` | `L2` |
|---|---|---|---|---|
| Presencia | ❌ | ✅ | ✅ | ✅ |
| Gap abre a | — | 7 min | 7 min | 7 min |
| Deduce el gap | ❌ | ✅ | ✅ | ✅ |
| Aborta por gap acumulado | — | ❌ | **35%** | **25%** |
| Aborta por **pantalla apagada** (`dpms off`) | — | ❌ | ❌ (acumula) | ✅ inmediato |
| Aborta por suspensión | — | ❌ | ✅ | ✅ |
| Aborta por kill del daemon | — | — | ✅ | ✅ |
| Crédito · aborto por **inactividad** | — | — | conserva | 0 si pantalla apagada o gap único ≥30% |
| Crédito · aborto **manual** | — | — | **conserva** | **conserva** |
| Deuda | — | ❌ | **5 min** +1/aborto (tope 20) | **8 min** +1/aborto (tope 25) |
| Bloquea | — | — | la actividad abortada | la actividad abortada |
| Pago | — | — | 1 min de foco = 1 min de deuda | ídem |
| Refinanciación | — | ❌ | **+8%** de lo acreditado, +2%/aborto (tope 40%) | **+15%**, +2%/aborto (tope 40%) |
| Base si el crédito fue 0 | — | — | `transcurrido` | `transcurrido` |
| Caducidad de la deuda | — | — | **15 días** | **7 días** |
| Crecimiento de la deuda | — | — | +5 min/día | +5 min/día |
| Al vencer | — | — | entrada **negativa** en métricas | ídem |
| Perdones | — | — | 1 | 0 |
| Puntaje al abortar | opcional | opcional | 1–7 obligatorio | 1–5 obligatorio |
| Notas al abortar | opcional | opcional | opcional | **obligatorias (cualquier aborto)** |

```
gap_ratio = gap_acumulado / target_de_la_sesión        (el target incluye el pending_extra)
```

**`L0` no tiene contrato**: no aborta, no endeuda, no exige puntaje. Es **medición honesta** y por eso es 100% configurable (umbral de gap, modo de descuento, avisos). `Off` es cronómetro puro: sin presencia, acredita todo.

**`L1` y `L2` son presets nuestros: se eligen, no se configuran.** Lo único elegible es el nivel y el término del contrato. El usuario **los ve** en la pantalla `Contrato` (solo lectura), para entender a qué se está comprometiendo.

### 6. Deuda de reparación (el "cooldown" con sentido)

Esperar no cuesta nada, y la app no puede hacer nada fuera de sí misma. La única palanca real es **qué sesiones permite**. Por eso el cooldown deja de ser un reloj y pasa a ser **trabajo pendiente**:

```
deuda = min(tope, base + 1 min × abortos_previos) + 5 min/día sin pagar
```

Ciclo de vida:

1. Abortás → se crea la deuda y **la actividad queda bloqueada** para nuevas sesiones contratadas.
2. Se paga **automáticamente**: cada minuto acreditado en **cualquier** sesión trackeada descuenta 1 min.
3. Crece **+5 min por día** sin pagar (calculado desde `created_at`, sin job de mantenimiento).
4. Al vencer (**L1 15 días · L2 7 días**) el remanente se registra como **entrada negativa** en las métricas y la deuda se cierra.

**Qué no se puede**: no hay "modo reparación" especial ni tiempo de espera. Se trabaja en otra cosa, o se refinancia.

### 7. Refinanciación

Pagar la deuda **comprándola con más compromiso**:

```
base          = acreditado ; si el aborto dejó el crédito en 0 → transcurrido
pending_extra = ratio × base                              (se ACUMULA)
nuevo_target  = target_normal + pending_extra
```

- El `pending_extra` **se suma al target de la próxima sesión de esa actividad** y **no se puede arrancar sin él**.
- Se **acumula** con cada aborto refinanciado, **tope 40%**, y **no caduca**.
- Se resetea recién al **completar** esa sesión.

Como la base es lo **trabajado** (no el target), pagar en proporción a lo que realmente hiciste. Y si un aborto severo te dejó el crédito en 0, la base pasa a ser el **tiempo transcurrido**: refinanciar un aborto grave sale **más caro**.

### 8. Puntaje y notas

| Cierre | Nivel | Puntaje | Notas |
|---|---|---|---|
| Completada | cualquiera | 1–10, opcional | opcional |
| Abortada | `L1` | **1–7, obligatorio** | opcional |
| Abortada | `L2` | **1–5, obligatorio** | **obligatorias** |
| Abortada | `L0` / `Off` | opcional | opcional |

Se guarda **`rating_scale`** junto al puntaje. Sin eso, un `5` en L2 y un `5` en L1 no son comparables y **Focus Quality queda roto**. Las métricas normalizan con `rating / scale`.

Los abortos manuales se separan por `aborted_with = USER`: *"abortaste 8 veces esta semana, 7 por decisión"*. Es un dato, no un juicio.

### 9. Actividades: `timer` vs `manual`

| `tracking_mode` | Tiempo manual | Si arranca el timer |
|---|---|---|
| `timer` | ❌ rechazado | **Contrato completo** |
| `manual` | ✅ permitido | **Contrato completo** |

La única diferencia es **si se puede cargar tiempo a mano**. Las restricciones aplican siempre que haya timer. La elección es **obligatoria al crear**, con o sin agenda.

### 10. Contrato de compromiso

```
commitment_contracts { level, term, started_at, ends_at, params_snapshot, checksum }
```

- **Subir** de nivel: inmediato, exige un término (días/semanas/meses; mínimo 1 día).
- **Bajar**: bloqueado hasta `ends_at`.
- `params_snapshot` congela **todos** los parámetros del nivel. Mientras el contrato esté activo, el daemon **usa el snapshot**, no `config.toml`, y la TUI los muestra bloqueados. Un `config.toml` distinto se ignora y se registra la discrepancia.

### 11. Notificaciones

- Umbrales de inactividad: **3 min** (suave) · **5 min** (fuerte) · **7 min** (el gap abre).
- Se disparan **en el instante** en que se cruza cada umbral (evento → daemon → notificación), nunca diferidas.
- Fin de bloque: notificación de escritorio + sonido + salto a la pantalla de descanso.

## Alternativas consideradas

- **Aborto manual gratuito**: haría que cancelar en el minuto 1 no cueste nada y el contrato sería decorativo. Descartada.
- **Cooldown de reloj**: pago pasivo; se escurre mirando el techo. Descartada.
- **Pomodoro sin bordes libres**: obligaría a abortar para irse, castigando el uso normal. Descartada.
- **Niveles configurables**: cada usuario se ablanda su propio castigo. Descartada (presets fijos).
- **Penalizar toda inactividad**: castiga leer y pensar. Descartada (gracia + umbral de 7 min).

## Consecuencias

- El motor de sesiones es una **máquina de estados pura y testeable** (Fase 4).
- Las métricas distinguen **observado vs acreditado**, `source`, `aborted_with` y `rating_scale`.
- `L0` es la puerta de entrada amable; `L1`/`L2` son el compromiso fuerte y visible.
- El escape real (navegador, teléfono) queda fuera del alcance: el contrato es **autoimpuesto** (`ADR-007`).

## Referencias

- `docs/vision.md` (3.3–3.6) · `ADR-002` · `ADR-008` · `docs/phases.md` (Fases 4 y 5)
