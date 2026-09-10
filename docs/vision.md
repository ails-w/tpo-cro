# 👁️ Documento de Visión y Especificación Funcional

**Proyecto:** Terminal Productivity Tracker (`tpt`)  
**Inspiración:** Monitor pasivo de tiempo y enfoque (local / TUI) + Sistema de Rutinas de Super Productivity  
**Stack Core:** Rust, Arch Linux (Wayland/Hyprland), `ratatui` + `crossterm`, `rusqlite` (WAL mode)  
**Estado:** Consolidado — decisiones D1–D20 (ver `docs/adr/`)

---

## 🚀 1. Propósito y Filosofía del Producto

`tpt` es un monitor pasivo de tiempo, enfoque y fatiga diseñado para desarrolladores y power users de Linux. Su objetivo es ofrecer un dashboard de productividad en terminal, **100% local**, con **huella de memoria mínima** (sin runtime async pesado) y control total por teclado.

*   **Privacidad Absoluta:** Cero telemetría, cero sincronización en nube, cero capturas de pantalla, cero keylogging.
*   **Minimalismo Eficiente:** Arquitectura desacoplada en Rust (Daemon en segundo plano sin runtime pesado + TUI cliente ejecutable a demanda + CLI de control).
*   **Terminal-Centric:** Detección de proyectos mediante rutas de trabajo (`$PWD`) y control total por teclado.

### Decisiones fundacionales (D1–D20)

| ID | Decisión |
|----|----------|
| D1 | 4º crate `tpt-cli` (control + report + import, cliente IPC) |
| D2 | Hexagonal: puertos (traits) en `tpt-core`, adaptadores en `tpt-daemon` |
| D3 | Idle tras `IdleSource`; sin linkear libwayland; fallback polling |
| D4 | IPC versionado (`protocol_version`) + socket `0600` |
| D5 | Suspensión: reloj monotónico + gap = idle/offline |
| D6 | Modos de reloj: **Pomodoro**, **Focus-contrato**, **Flowtime** |
| D7 | `+`/`-` en Pomodoro y descansos; jamás en Focus |
| D8 | Pomodoro sin pausa; abortar = ciclos reiniciados + sin racha + reflexión obligatoria |
| D9 | Focus: crédito 0 + cooldown 5 min + challenge; reflexión obligatoria |
| D10 | Cooldown 5 min visible en TUI |
| D11 | Challenge/contraseña + reflexión obligatoria en ambos modos |
| D12 | Inactividad: sin penalización por no-teclear; aviso informativo (300 s); gap = offline |
| D13 | Blocklist por tarea (apps prohibidas); switch de ventanas libre |
| D14 | Switch de tareas programadas en Pomodoro y Focus (sin penalización) |
| D15 | Analytics: observado vs acreditado; día trabajado + día calendario |
| D16 | Extras: productivo vs distracción, media de sesión, switches, racha |
| D17 | Retención modular por tiers (raw/hourly/daily) |
| D18 | BTRFS: nodatacow + checkpoint + incremental_vacuum + snapshots excluidos |
| D19 | Presets editables `25/5`, `50/10`, `52/15`, `90/15(20)`; corto ≤15, largo ≤20 (el clásico 52/17 queda clampado a 15) |
| D20 | Docs: `docs/` + `phases.md` + `development-plan.md`; `diagrams/` opcional |

---

## 🛠️ 2. Especificación Detallada de Funcionalidades Core

### 2.1. Motor de Captura y Categorización Pasiva (Daemon Engine)

1.  **Detección de Ventana Activa:**
    *   Escucha los eventos del socket IPC de Hyprland (`$XDG_RUNTIME_DIR/hypr/.../socket2.sock`) para identificar clase (`class`), título (`title`) y PID del proceso. *(D2: detrás de `WindowSource`)*
2.  **Tracking por Rutas Terminal ($PWD Exclusivo):**
    *   Si la ventana activa es un emulador de terminal (ej. Kitty, Alacritty, Foot), el daemon lee `/proc/<PID>/cwd`.
    *   Si la ruta coincide con un repositorio o directorio configurado, se auto-asigna el tiempo al **Proyecto** correspondiente, ignorando reglas genéricas.
3.  **Evaluación de Reglas Regex (No-IA):**
    *   Procesamiento secuencial de reglas en `config.toml` sobre `class`, `title` o `pwd`.
    *   Asignación instantánea a **Categorías** (ej. *Enfoque / Dev*, *Comunicación*, *Distracción*, *Utilidades*).
4.  **Detección de Inactividad (Idle Time):**
    *   Integración con `ext-idle-notify-v1` de Wayland (vía `hypridle` / helper) tras `IdleSource`. *(D3)*
    *   **La falta de input NUNCA penaliza** *(D12)*: solo dispara un aviso informativo tras el umbral (`warn_after_seconds`, default 300 s) y el gap se registra como `idle/offline` (no cuenta como productivo, no rompe racha, no reinicia ciclos).
    *   Al regresar, la TUI despliega un modal interrumpible: *"¿Monitoreo pausado: fue descanso o actividad offline?"*, permitiendo clasificar o descartar el bloque.
    *   **Suspensión (D5):** duraciones con reloj monotónico; el gap de suspensión se trata como idle/offline.

### 2.2. Visualización y Gráficos (Dashboard de Productividad)

1.  **Barra de Línea de Tiempo Diaria (24h Timeline Bar):**
    *   Barra horizontal continua con caracteres de bloque (`█`, `▓`, `▒`, `░`).
    *   Codificación de color por categoría (Verde = Enfoque, Rojo = Distracción, Azul = Comunicación, Gris = Idle).
2.  **Distribución Categórica / Proyectos (Bar Charts Horizontales):**
    *   Barras ordenadas de mayor a menor consumo, con porcentaje y horas/minutos exactos.
3.  **Métrica de Nivel de Enfoque (Focus Score):**
    *   Indicador de $0$ a $100\%$ según proporción entre tiempo productivo vs. distracciones e interrupciones. Fórmula configurable.
4.  **Breakdown de Sub-actividades (Context Switch Frequency):**
    *   Contador de "Cambios de Contexto" por hora (conmutación entre apps en intervalos < 2 min).

### 2.3. Temporizadores: Pomodoro, Focus y Flowtime (D6–D11)

#### Modo Pomodoro (estructurado, "arrancador")
*   Ciclos trabajo → descanso corto; cada $K$ ciclos, descanso largo.
*   **NO tiene pausa** *(D8)*: si no podés seguir, abortás.
*   **Ajustable con `+`/`-`** por minuto (step y límites configurables) en trabajo y descansos. *(D7)*
*   **Penalización por abortar:** reinicio de contador de ciclos; **no se guarda la racha de focus**; las métricas NO se tocan.
*   **Regla de integridad:** el pomodoro solo se completa si el tiempo productivo alcanza el target; el idle nunca lo completa solo.
*   **Reinicio de ciclos si no se termina** el pomodoro dentro del tiempo ajustado.

#### Modo Focus (contrato duro)
*   Duración fija e **inmutable**: sin `+`, sin `-`, sin pausa, sin stop.
*   Contrato = $X$ **minutos productivos** (idle no cuenta).
*   **Penalización por abortar** *(D9)*: crédito = 0, **cooldown bloqueante de 5 min** (visible en TUI, D10), **challenge/contraseña** para abortar, **reflexión obligatoria** (motivo escrito), impacto directo en métricas (`ABORTED_PENALIZED`).

#### Modo Flowtime (medir, no imponer)
*   Sesión de duración variable; se detiene cuando la atención se rompe.
*   Descanso proporcional al bloque trabajado (escala heurística).
*   Sin penalización (es medición).

#### Reglas transversales
*   **Challenge/contraseña + reflexión obligatoria en ambos modos** para abortar *(D11)*.
*   **Switch de tareas programadas** a mitad de sesión en Pomodoro y Focus, sin penalización *(D14)*: el tiempo se acredita a la tarea activa al cierre (o se parte en segmentos por tarea).
*   **Cooldown de 5 minutos** visible en la TUI tras un aborto Focus *(D10)*.

#### Presets de reloj (D19)
*   Presets editables antes de iniciar: `25/5`, `50/10`, `52/15`, `90/15 (largo 20)`.
*   **Regla dura:** descanso corto **≤15 min**, descanso largo **≤20 min**.

### 2.4. Gestión de Descansos Inteligentes (Fatigue Detection)

1.  **Notificación de Fatiga Acumulada:**
    *   Calcula el tiempo continuo real en estado de "Enfoque Profundo" (`is_productive = true`).
    *   Al alcanzar el límite configurado (ej. 50 min), la TUI lanza una alerta visual de fatiga.
2.  **Modo de Descanso Obligatorio / Pauta Visual (Take-a-Break Mode):**
    *   Modo opcional que toma la pantalla de la TUI mostrando un temporizador gigante en ASCII Art e instrucciones de descanso/ergonomía visual (regla 20-20-20).

### 2.5. Rutinas y Tareas Recurrentes (Integración Super Productivity)

1.  **Rutinas Diarias (Daily Routine Items):**
    *   Metas recurrentes (ej. *"Estudiar Rust - Meta: 60 min"*).
    *   Se auto-generan cada día y muestran una barra de progreso ($X / Y \text{ mins}$) combinando el tiempo registrado por el tracker pasivo con la tarea vinculada.
2.  **Consistencia de Rutinas (Habit Streak):**
    *   Métrica mensual de días cumplidos. Una sesión abortada **no acredita** al streak.

### 2.6. Analíticas y Reportes Históricos (Día / Semana / Mes)

Persistente en SQLite. Vistas por pestañas `[D]ía`, `[S]emana`, `[M]es`:

1.  **Métricas base (D15, D16):**
    *   **Tiempo invertido en horas** por rango (observado y acreditado).
    *   **Días trabajados en el mes** (días con actividad > umbral).
    *   **Promedios mostrando ambos denominadores:** por *día trabajado* y por *día calendario*.
    *   **Productivo vs distracción**, duración media de sesión, context switches, racha, sesiones abortadas.
2.  **Vista Semanal:** barras acumuladas Lunes a Domingo (enfoque vs distracción/descanso), ranking de proyectos, mejor día.
3.  **Vista Mensual:** heatmap estilo *GitHub Contribution Graph* con caracteres ANSI/Unicode.
4.  **Exportador CLI (tpt-cli, D1):**
    *   `tpt-cli report --range weekly --format json`
    *   `tpt-cli report --range monthly --format csv`

### 2.7. Inactividad y Blocklist (D12, D13)

1.  **Inactividad (D12):** la falta de input es un dato, no una infracción:
    *   Aviso informativo (no punitivo) tras `warn_after_seconds` (default 300 s): *"¿Seguís con la sesión? El tiempo idle no cuenta como productivo."*
    *   El tiempo idle se registra como `idle/offline` y se descuenta del acumulado productivo.
    *   Al volver, modal de clasificación (descanso vs actividad offline).
2.  **Blocklist por tarea (D13):**
    *   Cada tarea/proyecto declara **apps prohibidas** (más descriptivo que whitelist): al iniciar el bloque sabés exactamente qué NO podés abrir.
    *   **Cambiar de ventana entre apps no bloqueadas es libre (nunca se penaliza).**
    *   Abrir una app bloqueada → aviso → countdown **3–5 min** → si no la cerrás: penalización según modo (Pomodoro: ciclos · Focus: crédito 0 + cooldown 5 min + challenge). Máx **2 avisos/sesión**.

### 2.8. Datos y Persistencia (D17, D18)

*   **Tiers de retención modular:** `raw_events` (30 días) → `hourly_aggregates` (90 días) → `daily_aggregates` (permanente).
*   **BTRFS:** `chattr +C` (nodatacow) en el directorio de la DB antes de crearla; checkpoint `TRUNCATE` periódico; `auto_vacuum=INCREMENTAL` + `incremental_vacuum(N)` (nunca `VACUUM` completo); snapshots BTRFS excluyen la DB o usan backup consistente.

---

## 🔄 3. Migración de Datos desde Super Productivity (JSON)

Subcomando dedicado para importar tareas, historial de tiempo y rutinas desde un backup JSON de Super Productivity, con validación, `--dry-run` e idempotencia/dedup:

```bash
tpt-cli import --super-productivity ~/.config/superProductivity/save.json
```

---

## 🧭 4. Fuera de Alcance Inicial

- Interfaz gráfica (GUI).
- Microservicios / arquitectura distribuida.
- Soporte multi-plataforma (solo Linux/Arch + Wayland/Hyprland).
- Optimización prematura de rendimiento.
- Autenticación de red / multi-usuario.
- Apuestas monetarias o accountability social como penalización (se mantiene local).