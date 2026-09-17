# 👁️ Visión y Especificación Funcional — TPT

**Proyecto:** Terminal Productivity Tracker (`tpt`)
**Norte:** Super Productivity (actividades + tiempo) con un **motor de sesiones restrictivo** propio.
**Stack:** Rust · Arch Linux / Wayland (Hyprland) · `ratatui` + `crossterm` · `rusqlite` (WAL)
**Estado:** Pivote *timer-first* — decisiones D1–D20 revisadas (`docs/adr/`)

> **Cambio de rumbo (2026-09).** Se abandonó el tracking pasivo de ventanas (Hyprland `socket2`, `$PWD`, reglas regex) y la blocklist de apps. Motivo y alcance → `ADR-007-timer-first-pivot.md`.

---

## 1. Propósito y Filosofía

`tpt` es un gestor de actividades de terminal, **100% local**, que te obliga a comprometerte con contratos de tiempo explícitos y te devuelve una medida honesta de tus hábitos.

- **Privacidad absoluta:** cero telemetría, cero nube, cero capturas, cero keylogging.
- **Minimalismo eficiente:** daemon sin runtime async pesado + TUI a demanda + CLI.
- **El contrato es lo que duele:** el timer **no se pausa ni se acorta**. Solo se extiende o se cancela con costo.
- **Default amable:** medir no es castigar. Los castigos existen, son configurables y son **opt-in**.
- **Datos sobre opiniones:** la mejora se mide (rachas, promedios, tendencias), no se intuye.

### Lo que la app **no** hace

No espía qué apps usás. No categoriza ventanas. No bloquea programas. El compromiso es **autoimpuesto**: la app es el cronómetro honesto y el registro, no el guardia.

---

## 2. Decisiones Fundacionales (D1–D20)

| ID | Decisión |
|----|----------|
| D1 | 4º crate `tpt-cli` (control + report + import + daemon lifecycle) |
| D2 | Hexagonal: puertos (traits) en `tpt-core`, adaptadores en `tpt-daemon` |
| D3 | **Presencia** (idle/lock/suspend) vía hooks de `hypridle` + detección de salto de reloj; sin D-Bus ni libwayland |
| D4 | IPC versionado (`protocol_version`) + socket `0600` |
| D5 | Reloj monotónico + wall-clock; el salto de suspensión **no acredita** tiempo |
| D6 | Tres modos: **Flowtime**, **Pomodoro**, **Focus** (contrato duro) |
| D7 | **Contrato aditivo:** solo `+` (extender), nunca `-`. No se acorta una sesión |
| D8 | **Sin pausa** en Pomodoro y Focus. **Flowtime sí tiene pausa** |
| D9 | Abortar Focus = crédito 0 + cooldown 5 min + challenge + reflexión obligatoria |
| D10 | Cooldown de 5 min visible en la TUI |
| D11 | Challenge + reflexión obligatoria al **abortar**; opcional al completar |
| D12 | Inactividad: aviso informativo (3/5 min) y gracia de 3 min; por default **no castiga** |
| D13 | Presets ajustables: `25/5`, `50/10`, `75/12`, `90/15` (largo ≤20, corto ≤15) |
| D14 | Switch de actividad a mitad de sesión sin penalización: el tiempo se parte |
| D15 | Métricas: **observado vs acreditado**; promedios por día trabajado **y** por día calendario |
| D16 | **Niveles de estrictez** configurables (`Off`/`L0`/`L1`/`L2`) + **contrato de compromiso** por término |
| D17 | Actividades con **agenda opcional** (recurrencia) y **cola del día** |
| D18 | Tiempo **manual** e **importado** se etiquetan, y **suman al total** de métricas |
| D19 | SQLite sin tiers de agregación: métricas calculadas sobre `sessions` + `time_entries` |
| D20 | Docs en `docs/`; sin carpeta de diagramas por defecto |

---

## 3. Funcionalidades Core

### 3.1 Actividades y Proyectos

- Una **actividad** es la unidad a la que se acredita el tiempo: `nombre`, `proyecto` (opcional), `tags`, `target_minutes` (opcional), `schedule` (opcional).
- El **schedule no es obligatorio**. Una actividad sin agenda existe y se usa ad-hoc.
- CRUD completo desde la TUI y el CLI.

### 3.2 Agenda y Cola del Día

- Reglas de recurrencia: **lunes a viernes**, **días específicos**, **toda la semana**, o **rango de fechas** (calendario).
- Cada día la app arma la **cola del día** combinando agenda + pendientes.
- Al **completar** una actividad, pasa a historial y puede entrar la siguiente de la cola.

### 3.3 Motor de Sesiones (el contrato)

- **Toda sesión está asociada a una actividad.** No existe timer sin actividad.
- **Flowtime:** cuenta libre, **con pausa**, sin contrato.
- **Pomodoro:** ciclos trabajo → descanso; presets ajustables; durante trabajo solo `+`; durante descanso solo *saltear* o *terminar*.
- **Focus:** contrato duro e inmutable; solo `+` para extender.
- **Cerrar la TUI no aborta.** La sesión vive en el daemon.
- **Switch de actividad** a mitad de sesión: el tiempo se parte y el contrato continúa.

Detalle → `ADR-004-clock-modes-and-penalties.md`.

### 3.4 Presencia, Inactividad y Penalizaciones

- Señales de presencia: **teclado idle**, **bloqueo de pantalla**, **suspensión/apagado**.
- Fuente primaria: **hooks de `hypridle`**; backstop: **salto de reloj** (D3/D5).
- Escalada informativa a los 3 y 5 min; a los 7 min el gap **deja de acreditarse**.
- **Niveles de estrictez** (`Off`/`L0`/`L1`/`L2`) configurables, con **contrato de compromiso** por término para evitar el downgrade caprichoso.

Detalle → `ADR-008-presence-and-penalties.md`.

### 3.5 Tiempo Manual

- Se puede **agregar tiempo manual** a cualquier actividad (trabajo hecho fuera del timer).
- Se guarda como `TimeEntry { source: Manual }`, **etiquetado visualmente**, y **suma al total** de las métricas.
- Nunca altera una sesión en curso.

### 3.6 Reflexión y Notas

- Al salir de una sesión (abortar o completar) se ofrece un **rating de foco 1–10** y **notas**.
- **Ambos son opcionales e independientes:** una sesión puede tener solo nota, solo puntaje, ambos, o nada.
- Se navegan en una vista **`Historial`** agrupada por fecha, filtrable por actividad y rating.
- Retención de notas configurable: se eliminan tras `notes_retention_days` (0 = nunca).

### 3.7 Métricas

- **Heatmap anual** de actividad (estilo GitHub) por día.
- **Línea temporal** de sesiones y minutos de foco, con rangos `2 semanas` / `1 mes` / `Máx`.
- **Jerarquía Año → Mes → Semana → Día** con rangos horarios, conteo de actividades y tiempo trabajado.
- **Focus Quality** (promedio de rating) por actividad y franja horaria.
- Total, días trabajados, meses trabajados, promedios por **día trabajado** y por **día calendario**.
- Rachas con política *never miss twice*.

### 3.8 Migración desde Super Productivity

```bash
tpt-cli import --super-productivity ~/.config/superProductivity/save.json [--dry-run]
```

- Importa `project`, `task`, `tag` y **`timeSpentOnDay`** (ms).
- **Idempotente** por `external_id` (el `id` de SP).
- El tiempo importado se etiqueta como `Imported` y suma al total.

---

## 4. Fuera de Alcance

- Tracking de ventanas, categorización automática y blocklist de apps.
- Interfaz gráfica (GUI).
- Microservicios / arquitectura distribuida.
- Soporte multi-plataforma (solo Linux/Arch + Wayland/Hyprland).
- Autenticación de red / multi-usuario.
