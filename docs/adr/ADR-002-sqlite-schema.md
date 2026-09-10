# ADR-002: Esquema Relacional de Base de Datos (SQLite)

- **Estado**: Aceptado
- **Fecha**: 2026-09-10

## Contexto

Se requiere almacenar registros crudos de tiempo, categorías, rutinas recurrentes, bloques de apps prohibidas e historial de sesiones de enfoque manteniendo consultas eficientes para rangos de días, semanas y meses.

## Decisión

Esquema SQLite inicial (versión de esquema + migración en Fase 2):

```sql
-- Categorías principales
CREATE TABLE IF NOT EXISTS categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    color_hex TEXT NOT NULL,
    is_productive BOOLEAN DEFAULT 1
);

-- Tareas o Rutinas Recurrentes (Estilo Super Productivity)
CREATE TABLE IF NOT EXISTS recurring_tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    category_id INTEGER,
    target_duration_minutes INTEGER DEFAULT 0,
    is_active BOOLEAN DEFAULT 1,
    FOREIGN KEY (category_id) REFERENCES categories(id)
);

-- Apps prohibidas por tarea (D13: blocklist por tarea)
CREATE TABLE IF NOT EXISTS task_blocked_apps (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id INTEGER NOT NULL,
    app_name TEXT NOT NULL,
    FOREIGN KEY (task_id) REFERENCES recurring_tasks(id),
    UNIQUE(task_id, app_name)
);

-- Logs de Actividad de Ventanas (Daemon Tracking)
CREATE TABLE IF NOT EXISTS window_activity_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    app_name TEXT NOT NULL,
    window_title TEXT,
    working_directory TEXT,
    category_id INTEGER,
    start_time DATETIME NOT NULL,
    end_time DATETIME NOT NULL,
    duration_seconds INTEGER NOT NULL,
    is_idle BOOLEAN DEFAULT 0,
    FOREIGN KEY (category_id) REFERENCES categories(id)
);

-- Bloques de Sesión Pomodoro / Enfoque
CREATE TABLE IF NOT EXISTS focus_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    mode TEXT NOT NULL CHECK(mode IN ('POMODORO', 'FOCUS', 'FLOWTIME')),
    start_time DATETIME NOT NULL,
    end_time DATETIME,
    target_duration_seconds INTEGER NOT NULL,
    actual_duration_seconds INTEGER DEFAULT 0,
    productive_duration_seconds INTEGER DEFAULT 0,
    status TEXT CHECK(status IN ('COMPLETED', 'ABORTED_PENALIZED', 'ABORTED', 'IN_PROGRESS', 'SUSPENDED_IDLE')) NOT NULL,
    recurring_task_id INTEGER,
    reflection TEXT,
    aborted_with TEXT CHECK(aborted_with IN ('USER', 'BLOCKED_APP', 'IDLE')) DEFAULT 'USER',
    FOREIGN KEY (recurring_task_id) REFERENCES recurring_tasks(id)
);

-- Agregados para analíticas (D17: retención por tiers)
CREATE TABLE IF NOT EXISTS daily_aggregates (
    day TEXT NOT NULL,               -- YYYY-MM-DD
    productive_seconds INTEGER DEFAULT 0,
    distraction_seconds INTEGER DEFAULT 0,
    idle_seconds INTEGER DEFAULT 0,
    total_seconds INTEGER DEFAULT 0,
    sessions_completed INTEGER DEFAULT 0,
    sessions_aborted INTEGER DEFAULT 0,
    context_switches INTEGER DEFAULT 0,
    UNIQUE(day)
);

CREATE TABLE IF NOT EXISTS hourly_aggregates (
    hour TEXT NOT NULL,              -- YYYY-MM-DDTHH
    productive_seconds INTEGER DEFAULT 0,
    distraction_seconds INTEGER DEFAULT 0,
    idle_seconds INTEGER DEFAULT 0,
    total_seconds INTEGER DEFAULT 0,
    UNIQUE(hour)
);

-- Índices de Rendimiento para Analíticas Históricas
CREATE INDEX IF NOT EXISTS idx_activity_start_time ON window_activity_logs(start_time);
CREATE INDEX IF NOT EXISTS idx_activity_category ON window_activity_logs(category_id);
CREATE INDEX IF NOT EXISTS idx_focus_sessions_start ON focus_sessions(start_time);
```

## Alternativas consideradas

- **Sin agregados (solo raw)**: consultas lentas en rangos largos y DB sin límite.
- **Esquema polimórfico por modo**: complejidad innecesaria; un campo `mode` + CHECK es suficiente.

## Consecuencias

- `focus_sessions` gana campos de modo, productivo vs observado, reflexión y causa de aborto (D9/D11/D15).
- Los agregados permiten retención modular (raw 30 días, hourly 90, daily permanente) — ver `ADR-006-data-retention-btrfs.md`.
- La blocklist por tarea vive en `task_blocked_apps` (D13).

## Referencias

- `docs/vision.md` (2.3, 2.6, 2.7, 2.8) · `ADR-006-data-retention-btrfs.md`