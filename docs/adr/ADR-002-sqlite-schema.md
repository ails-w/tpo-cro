# ADR-002: Esquema SQLite

- **Estado**: Aceptado
- **Fecha**: 2026-09-14 · *(reemplaza el esquema del diseño con tracking de ventanas)*

## Contexto

Sin tracking pasivo, el volumen de escritura es **bajísimo** (decenas de filas por día). Pero el core de la app pasó a ser **métricas**: agregaciones por día/semana/mes, filtros por actividad y proyecto, promedios con dos denominadores y tendencias. Eso exige consultas, no archivos que se reescriben enteros.

## Decisión

**SQLite** para datos, **TOML** para configuración, **JSON** solo para import/export. Sin tablas de agregados ni tiers de retención.

```sql
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA foreign_keys = ON;
PRAGMA auto_vacuum = INCREMENTAL;

-- Proyectos
CREATE TABLE projects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    color_hex TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Actividades (la unidad a la que se acredita tiempo)
CREATE TABLE activities (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    project_id INTEGER REFERENCES projects(id) ON DELETE SET NULL,
    target_minutes INTEGER,
    schedule_rule TEXT,               -- JSON; NULL = sin agenda (permitido)
    archived INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Etiquetas
CREATE TABLE tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE activity_tags (
    activity_id INTEGER NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (activity_id, tag_id)
);

-- Sesiones del timer
CREATE TABLE sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    activity_id INTEGER NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
    mode TEXT NOT NULL CHECK(mode IN ('FLOWTIME','POMODORO','FOCUS')),
    target_seconds INTEGER,           -- NULL en Flowtime
    elapsed_seconds INTEGER NOT NULL DEFAULT 0,
    credited_seconds INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL CHECK(status IN
        ('RUNNING','BREAK','SUSPENDED','COMPLETED','ABORTED','ABORTED_PENALIZED')),
    cycle INTEGER NOT NULL DEFAULT 1,
    strictness TEXT NOT NULL CHECK(strictness IN ('OFF','L0','L1','L2')),
    started_at TEXT NOT NULL,
    ended_at TEXT,
    aborted_with TEXT CHECK(aborted_with IN
        ('USER','IDLE','LOCKED','SUSPENDED','DAEMON_KILLED')),
    rating INTEGER CHECK(rating BETWEEN 1 AND 10),   -- opcional
    reflection TEXT                                   -- opcional, independiente del rating
);

-- Entradas de tiempo (manual / derivadas de sesión / importadas)
CREATE TABLE time_entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    activity_id INTEGER NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
    source TEXT NOT NULL CHECK(source IN ('MANUAL','SESSION','IMPORTED')),
    seconds INTEGER NOT NULL CHECK(seconds >= 0),
    day TEXT NOT NULL,                -- YYYY-MM-DD local
    note TEXT,
    session_id INTEGER REFERENCES sessions(id) ON DELETE SET NULL,
    external_id TEXT UNIQUE,          -- id de Super Productivity → dedupe idempotente
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Extensiones (+) aplicadas a una sesión: auditoría del contrato aditivo
CREATE TABLE session_additions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    seconds INTEGER NOT NULL CHECK(seconds > 0),
    at TEXT NOT NULL
);

-- Gaps de presencia detectados durante una sesión
CREATE TABLE session_gaps (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK(kind IN ('IDLE','LOCKED','SUSPENDED')),
    seconds INTEGER NOT NULL CHECK(seconds >= 0),
    deducted_seconds INTEGER NOT NULL CHECK(deducted_seconds >= 0),
    strictness TEXT NOT NULL,
    at TEXT NOT NULL
);

-- Contrato de compromiso con el nivel de estrictez
CREATE TABLE commitment_contracts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    level TEXT NOT NULL CHECK(level IN ('OFF','L0','L1','L2')),
    term_seconds INTEGER NOT NULL,
    started_at TEXT NOT NULL,
    ends_at TEXT NOT NULL,
    checksum TEXT NOT NULL,           -- detecta edición a mano de la DB
    closed_at TEXT
);

CREATE INDEX idx_activities_project ON activities(project_id);
CREATE INDEX idx_activities_archived ON activities(archived);
CREATE INDEX idx_entries_activity_day ON time_entries(activity_id, day);
CREATE INDEX idx_entries_day ON time_entries(day);
CREATE INDEX idx_entries_source ON time_entries(source);
CREATE INDEX idx_sessions_activity ON sessions(activity_id, started_at);
CREATE INDEX idx_sessions_started ON sessions(started_at);
CREATE INDEX idx_gaps_session ON session_gaps(session_id);
```

### Reglas de datos

- **`rating` y `reflection` son independientes y opcionales.** Una sesión puede tener solo nota, solo puntaje, ambos o nada.
- **El historial se agrupa por fecha** derivada de `started_at` / `time_entries.day`; no hace falta una tabla de notas separada.
- **`credited_seconds` es la base del contrato**; `elapsed_seconds` es el reloj de pared. Nunca se confunden.
- **`time_entries.source`** distingue procedencia; los tres sources **suman al total** de métricas (D18).
- **`external_id UNIQUE`** es lo que hace idempotente el import de Super Productivity.

## Alternativas consideradas

- **JSON como almacenamiento primario**: más simple al inicio, pero obliga a escribir agregación, atomicidad y locking a mano. Con métricas como core, SQL gana. El JSON queda para import/export.
- **Tablas de agregados (`daily_aggregates` / `hourly_aggregates`)**: eran necesarias con el flood de eventos de ventanas. Sin tracking, se calculan on-the-fly.
- **Tiers de retención (raw/hourly/daily)**: el esquema anterior los necesitaba por el volumen de logs. Ya no aplica. La única retención que queda es **notas** (`notes_retention_days`, configurable).

## Consecuencias

- Consultas de métricas = una query (`GROUP BY`), no código de agregación propio.
- La DB queda chica y predecible; si algún día pesa, se agrega cache materializado **sin migrar el modelo**.
- La DB vive en BTRFS: aplicar `chattr +C` al directorio **antes** de crearla (ver `docs/architecture.md`).

## Referencias

- `docs/vision.md` (3.7) · `docs/architecture.md` (Modelo de Datos) · `ADR-007` · `ADR-004`
