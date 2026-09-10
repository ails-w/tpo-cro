# 👁️ Documento de Visión y Especificación Funcional

**Proyecto:** Terminal Productivity Tracker (`tpt`)  
**Inspiración:** Fork conceptual de Rize.io (TUI / Local) + Sistema de Rutinas de Super Productivity  
**Stack Core:** Rust, Arch Linux (Wayland/Hyprland), `ratatui` + `crossterm`, `rusqlite` (WAL mode)

---

## 🚀 1. Propósito y Filosofía del Producto

`tpt` es un monitor pasivo de tiempo, enfoque y fatiga diseñado para desarrolladores y power users de Linux. Su objetivo es replicar y adaptar la experiencia visual de **Rize.io** a un entorno **100% local, impulsado por terminal y con huella de memoria mínima (<15 MB RAM)**.

*   **Privacidad Absoluta:** Cero telemetría, cero sincronización en nube, cero capturas de pantalla, cero keylogging.
*   **Minimalismo Eficiente:** Arquitectura desacoplada en Rust (Daemon en segundo plano sin runtime pesado + TUI cliente ejecutable a demanda).
*   **Terminal-Centric:** Detección de proyectos mediante rutas de trabajo (`$PWD`) y control total por teclado.

---

## 🛠️ 2. Especificación Detallada de Funcionalidades Core

### 2.1. Motor de Captura y Categorización Pasiva (Daemon Engine)
1.  **Detección de Ventana Activa:**
    *   Escucha los eventos del socket IPC de Hyprland (`$XDG_RUNTIME_DIR/hypr/.../socket2.sock`) para identificar clase (`class`), título (`title`) y PID del proceso.
2.  **Tracking por Rutas Terminal ($PWD Exclusivo):**
    *   Si la ventana activa es un emulador de terminal (ej. Kitty, Alacritty, Foot), el daemon lee `/proc/<PID>/cwd`.
    *   Si la ruta coincide con un repositorio o directorio configurado, se auto-asigna el tiempo al **Proyecto** correspondiente, ignorando reglas genéricas.
3.  **Evaluación de Reglas Regex (No-IA):**
    *   Procesamiento secuencial de reglas en `config.toml` sobre `class`, `title` o `pwd`.
    *   Asignación instantánea a **Categorías** (ej. *Enfoque / Dev*, *Comunicación*, *Distracción*, *Utilidades*).
4.  **Detección de Inactividad (Idle Time):**
    *   Integración con el protocolo `ext-idle-notify-v1` de Wayland (vía `hypridle` / libwayland).
    *   Al superar $N$ minutos de inactividad, el daemon pausa el registro de tiempo.
    *   Al regresar, la TUI despliega un modal interrumplible: *"¿Monitoreo pausado: fue descanso o actividad offline?"*, permitiendo clasificar o descartar el bloque.

---

### 2.2. Visualización y Gráficos (Dashboard Inspirado en Rize.io)

La TUI replica el lenguaje visual de Rize.io mediante componentes ANSI/ASCII de `ratatui`:

**Timeline Diaria:** Barra horizontal que muestre por horario horas de productividad, descanso, etc.

1.  **Barra de Línea de Tiempo Diaria (24h Timeline Bar):**
    *   Barra horizontal continua utilizando caracteres de bloque (`█`, `▓`, `▒`, `░`).
    *   Codificación de color por categoría (ej. Verde = Enfoque, Rojo = Distracción, Azul = Comunicación, Gris = Idle).
2.  **Distribución Categórica / Proyectos (Bar Charts Horizontales):**
    *   Barras horizontales ordenadas de mayor a menor consumo acumulado, con desglose de porcentaje y horas/minutos exactos.
3.  **Métrica de Nivel de Enfoque (Focus Score):**
    *   Indicador numérico de $0$ a $100\%$ calculado en tiempo real según la proporción entre tiempo en categorías productivas vs. distracciones e interrupciones.
4.  **Breakdown de Sub-actividades (Context Switch Frequency):**
    *   Contador de "Cambios de Contexto" por hora. Detecta fragmentación de atención al conmutar entre aplicaciones en intervalos menores a 2 minutos.

---

### 2.3. Temporizador de Enfoque y Pomodoro Estricto
1.  **Sesiones Rize Flexibles:** Mide ráfagas reales de enfoque continuo en lugar de forzar bloques rígidos.
2.  **Regla Anti-Abandono Estricta:**
    *   Si el usuario inicia un bloque de enfoque objetivo (ej. 45 min) y decide cancelarlo prematuramente:
        *   El estado se persiste en SQLite como `ABORTED_PENALIZED`.
        *   El contador de progreso de la sesión actual se **invalida por completo** (0 minutos otorgados a la meta).
        *   El estado del temporizador se mantiene en SQLite para prevenir que se resetee cerrando y reabriendo la TUI.

---

### 2.4. Gestión de Descansos Inteligentes (Fatigue Detection)
1.  **Notificación de Fatiga Acumulada:**
    *   Calcula el tiempo continuo real en estado de "Enfoque Profundo" (`is_productive = true`).
    *   Al alcanzar el límite configurado (ej. 50 min), la TUI lanza una alerta visual de fatiga.
2.  **Modo de Descanso Obligatorio / Pauta Visual (Take-a-Break Mode):**
    *   Modo opcional que toma la pantalla de la TUI mostrando un temporizador gigante en ASCII Art e instrucciones de descanso/ergonomía visual.

---

### 2.5. Rutinas y Tareas Recurrentes (Integración Super Productivity)

Combina el *Time Tracking* pasivo tipo Rize con la *Gestión de Tareas Recurrentes* de Super Productivity:

1.  **Rutinas Diarias (Daily Routine Items):**
    *   Modelado de metas recurrentes (ej. *"Estudiar Rust - Meta: 60 min"*, *"Revisión PRs - Meta: 30 min"*).
    *   Se auto-generan cada día y muestran una barra de progreso ($X / Y \text{ mins}$) combinando el tiempo registrado por el tracker pasivo con la tarea vinculada.
2.  **Consistencia de Rutinas (Habit Streak):**
    *   Métrica mensual que indica cuántos días del mes se cumplió la meta de tiempo de cada rutina.

---

### 2.6. Analíticas y Reportes Históricos (Día / Semana / Mes)

Permite cambiar la vista de la TUI mediante pestañas de rango (`[D]ía`, `[S]emana`, `[M]es`):

1.  **Vista Semanal (Weekly Overview):**
    *   Gráfico de barras acumuladas de Lunes a Domingo comparando horas enfocadas vs. horas de distracción/descanso.
    *   Ranking semanal de proyectos e identificador de días de mayor productividad.
2.  **Vista Mensual (Monthly Heatmap):**
    *   Matriz visual de 30/31 días estilo *GitHub Contribution Graph* utilizando caracteres ANSI/Unicode para indicar la intensidad de tiempo enfocado por día.
3.  **Exportador CLI (Reporting Engine):**
    *   Generación de reportes locales sin necesidad de abrir la TUI:
        *   `tpt-cli report --range weekly --format json`
        *   `tpt-cli report --range monthly --format csv`

## 🔄 3. Migración de Datos desde Super Productivity (JSON)

Subcomando dedicado para importar tareas, historial de tiempo y rutinas desde un backup JSON de Super Productivity:

```bash
tpt-cli import --super-productivity ~/.config/superProductivity/save.json
