# ADR-002: Esquema SQLite

- **Estado**: Aceptado
- **Fecha**: 2026-09-14 · *(reemplaza el esquema del diseño con tracking de ventanas)*

## Contexto

Sin tracking pasivo, el volumen de escritura es **bajísimo** (decenas de filas por día). Pero el core de la app pasó a ser **métricas**: agregaciones por día/semana/mes, filtros por actividad y proyecto, promedios con dos denominadores y tendencias. Eso exige consultas, no archivos que se reescriben enteros.

Además, el contrato (`ADR-004`) introduce **deuda de reparación**, **refinanciación** y **penalizaciones de tiempo** que hay que persistir sin agregar mantenimiento.

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
    tracking_mode TEXT NOT NULL CHECK(tracking_mode IN ('TIMER','MANUAL')),
    pending_extra_ratio REAL NOT NULL DEFAULT 0,   -- refinanciación acumulada (topa en 0.40)
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
        ('RUNNING','BREAK','AWAITING','SUSPENDED','COMPLETED','ABORTED','ABORTED_PENALIZED')),
    cycle INTEGER NOT NULL DEFAULT 1,
    total_cycles INTEGER,             -- N ciclos del Pomodoro; NULL en otros modos
    strictness TEXT NOT NULL CHECK(strictness IN ('OFF','L0','L1','L2')),
    started_at TEXT NOT NULL,
    ended_at TEXT,
    aborted_with TEXT CHECK(aborted_with IN
        ('USER','IDLE','SCREEN_OFF','SUSPENDED','DAEMON_KILLED')),
    accumulated_gap_seconds INTEGER NOT NULL DEFAULT 0,
    gap_ratio REAL NOT NULL DEFAULT 0,
    rating INTEGER CHECK(rating BETWEEN 1 AND 10),    -- opcional
    rating_scale INTEGER CHECK(rating_scale IN (5,7,10)),
    reflection TEXT                                    -- opcional, independiente del rating
);

-- Entradas de tiempo (manual / derivadas de sesión / importadas / penalizaciones)
CREATE TABLE time_entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    activity_id INTEGER NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
    source TEXT NOT NULL CHECK(source IN ('MANUAL','SESSION','IMPORTED','PENALTY')),
    seconds INTEGER NOT NULL,          -- negativo SOLO cuando source = 'PENALTY'
    day TEXT NOT NULL,                 -- YYYY-MM-DD local
    note TEXT,
    session_id INTEGER REFERENCES sessions(id) ON DELETE SET NULL,
    external_id TEXT UNIQUE,           -- id de Super Productivity → dedupe idempotente
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    CHECK (source = 'PENALTY' OR seconds >= 0)
);

-- Deuda de reparación (ADR-004 §6)
CREATE TABLE debts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    activity_id INTEGER NOT NULL REFERENCES activities(id) ON DELETE CASCADE,
    level TEXT NOT NULL CHECK(level IN ('L1','L2')),
    base_seconds INTEGER NOT NULL,     -- monto al crearse
    growth_per_day_seconds INTEGER NOT NULL DEFAULT 300,   -- +5 min/día
    cap_seconds INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    due_at TEXT NOT NULL,              -- L1: +15 días · L2: +7 días
    paid_seconds INTEGER NOT NULL DEFAULT 0,
    closed_at TEXT,
    closed_reason TEXT CHECK(closed_reason IN ('PAID','REFINANCED','EXPIRED'))
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
    kind TEXT NOT NULL CHECK(kind IN ('IDLE','SCREEN_OFF','SUSPENDED')),
    seconds INTEGER NOT NULL CHECK(seconds >= 0),
    deducted_seconds INTEGER NOT NULL CHECK(deducted_seconds >= 0),
    qualified INTEGER NOT NULL DEFAULT 0,   -- 1 si superó min_gap_seconds
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
    params_snapshot TEXT NOT NULL,     -- JSON con TODOS los parámetros del nivel (ADR-004 §10)
    checksum TEXT NOT NULL,            -- cubre level + term + params_snapshot
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
CREATE INDEX idx_debts_open ON debts(activity_id, closed_at);
```

### Reglas de datos

- **`tracking_mode` es obligatorio al crear** (`ADR-004` §9). Gobierna si se aceptan entradas `MANUAL`; el contrato aplica siempre que haya timer.
- **`rating` y `reflection` son independientes y opcionales.** `rating_scale` (5/7/10) hace comparables los puntajes entre niveles — sin él, **Focus Quality queda roto**.
- **`gap_ratio = accumulated_gap_seconds / target_seconds`.** Es el número que dispara el aborto y el que la TUI muestra en vivo.
- **`credited_seconds` es la base del contrato**; `elapsed_seconds` es el reloj de pared. Nunca se confunden.
- **`source = 'PENALTY'`** es el único caso con `seconds` negativo: es el cobro de una deuda vencida. La vista de métricas lo muestra como línea propia, no mezclado en el total.
- **El contador de abortos NO se persiste**: se calcula contando `sessions` con estado abortado de esa actividad desde el inicio del contrato.
- **El crecimiento de la deuda y su vencimiento se calculan**, no se escriben: nada de jobs diarios.

## Alternativas consideradas

- **JSON como almacenamiento primario**: obliga a escribir agregación, atomicidad y locking a mano. Con métricas como core, SQL gana.
- **Tabla separada para penalizaciones**: evita el `CHECK` condicional en `time_entries`, pero duplica el camino de lectura de métricas. Se prefirió una sola tabla con `source`.
- **Persistir el contador de abortos**: sería un contador mutable que puede desincronizarse. Contarlo desde `sessions` es siempre consistente.
- **Job diario para el crecimiento de la deuda**: escrituras periódicas y estado duplicado. Calcularlo desde `created_at` es exacto y no tiene mantenimiento.

## Consecuencias

- Consultas de métricas = una query (`GROUP BY`), no código de agregación propio.
- La DB queda chica y predecible; si algún día pesa, se agrega cache materializado **sin migrar el modelo**.
- La DB vive en BTRFS: aplicar `chattr +C` al directorio **antes** de crearla (ver `docs/architecture.md`).

## Referencias

- `docs/vision.md` (3.7) · `docs/architecture.md` (Modelo de Datos) · `ADR-004` · `ADR-007`
