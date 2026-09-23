# Fase 02 — Store SQLite

> Conceptos de la fase 2 del proyecto.
> Log de la fase → `docs/progress-log/phase-02-store.md`

---

## WAL con un hilo escritor dedicado

### Qué es

El daemon **no escribe SQL**. Arma el registro, le estampa un id y lo encola en un canal; un hilo aparte, dueño de la única conexión de escritura, aplica el SQL en serie. Las lecturas usan **otra** conexión, y WAL permite que convivan.

### Qué problema resuelve

El loop de sesión no puede quedarse esperando al disco. Si el timer depende de que un `INSERT` termine, un checkpoint o un disco lento se convierten en tiempo mal contado. Y en este producto, el tiempo es el compromiso.

### Para qué sirve en este proyecto

Es la diferencia entre "el daemon es la restricción" (`ADR-001`) y "el daemon es un cuello de botella". El loop sigue contando mientras el disco hace lo suyo; el único punto donde se espera es explícito: `flush()`.

### Cómo se usa

```rust
// crates/tpt-daemon/src/adapters/sqlite/mod.rs — el id se asigna acá, sin I/O
fn save_session(&mut self, session: &Session) -> Result<i64, StoreError> {
    let mut record = session.clone();
    let id = self.allocate_id(RecordKind::Session, record.id);
    record.id = Some(id);
    self.enqueue(WriteCommand::SaveSession(record))?;   // sync_channel(1024)
    Ok(id)                                              // vuelve ya
}
```

```rust
// crates/tpt-daemon/src/adapters/sqlite/writer.rs — el hilo, sin cortocircuito
while let Ok(command) = receiver.recv() {
    match command {
        WriteCommand::SaveSession(session) => {
            record(&mut first_error, mapping::upsert_session(&connection, &session));
        }
        // ...
        WriteCommand::Flush(reply) => {
            let _ = reply.send(first_error.take().map_or(Ok(()), Err));
        }
        WriteCommand::Shutdown => break,
    }
}
```

### Error común

Dos, y los dos son silenciosos:

1. **Canal ilimitado.** `mpsc::channel` no tiene tope: si el escritor se traba, la cola crece sin límite y el daemon se come la memoria. Se usa `sync_channel(1024)`: la cola llena aplica contrapresión en vez de crecer.
2. **Cortocircuitar al primer error.** Descartar todo lo que venga después de una falla transitoria convierte un error en una sesión entera perdida. Acá el hilo registra el primer error, **sigue procesando**, y `flush()` lo reporta y lo limpia.

### Referencias

- `docs/adr/ADR-001-architecture-ipc-persistence.md` (Persistencia) · `docs/architecture.md` (Persistencia y BTRFS)

---

## Migraciones y versionado de esquema

### Qué es

`PRAGMA user_version` es un entero en el header del archivo de la DB que nosotros usamos como número de versión del esquema. Al abrir: si es `0`, se aplica el esquema v1 y se sella `1`; si es mayor al que conocemos, se rechaza.

### Qué problema resuelve

Un esquema que evoluciona necesita saber sobre qué versión está parado. Y el DDL de `ADR-002` usa `CREATE TABLE` pelado, sin `IF NOT EXISTS`: si una migración queda a medias, el próximo arranque falla con "table already exists" y la DB queda inabrible.

### Para qué sirve en este proyecto

Hoy hay una sola versión. La estructura está para que la Fase 3+ (CRUD, tags, cola) pueda agregar tablas o columnas sin borrar la DB de nadie.

### Cómo se usa

```rust
// crates/tpt-daemon/src/adapters/sqlite/schema.rs
pub(super) const SCHEMA_VERSION: i64 = 1;

// migración atómica: o queda entera, o no queda nada
connection.execute_batch("BEGIN IMMEDIATE;")?;
connection.execute_batch(DDL)?;          // 10 tablas + 9 índices de ADR-002
connection.execute_batch(&format!("PRAGMA user_version = {SCHEMA_VERSION};"))?;
connection.execute_batch("COMMIT;")?;
```

### Error común

**Fijar `auto_vacuum` tarde.** Va **antes** del primer `CREATE TABLE`; si la DB ya tiene tablas, SQLite exige un `VACUUM` completo para cambiarlo — y `VACUUM` reescribe todo el archivo, que es exactamente lo que no querés en BTRFS (`architecture.md`). El orden correcto es: `auto_vacuum` → `journal_mode` → DDL.

### Referencias

- `docs/adr/ADR-002-sqlite-schema.md` · `docs/architecture.md` (Persistencia y BTRFS)

---

## Derivar estado por cálculo en vez de escribirlo

### Qué es

La deuda de reparación no tiene job de mantenimiento. Se **calcula** cada vez que se lee, a partir de `created_at` y `due_at`, con una función pura del dominio.

### Qué problema resuelve

Un contador persistido se desincroniza en cuanto una escritura falla o el daemon estuvo apagado. Y un job diario agrega mantenimiento: hay que decidir cuándo corre, qué pasa si no corrió, y cómo recuperar los días perdidos. Calcular no tiene ninguno de esos problemas: es exacto en cualquier momento, incluso si la app estuvo cerrada dos semanas.

### Para qué sirve en este proyecto

`ADR-002` ya lo fija como regla ("el crecimiento de la deuda y su vencimiento se calculan, no se escriben: nada de jobs diarios"). Es también la razón por la que el contador de abortos no se persiste: se cuenta.

### Cómo se usa

```rust
// crates/tpt-core/src/domain/debt.rs
pub fn outstanding_seconds(&self, now: i64) -> i64 {
    if self.closed_at.is_some() { return 0; }
    let until = now.min(self.due_at);                       // no crece después de vencer
    let days = (until - self.created_at).max(0) / 86_400;
    let base = self.base_seconds.min(self.cap_seconds);     // el tope aplica a la base
    (base + self.growth_per_day_seconds * days - self.paid_seconds).max(0)
}

pub fn is_expired(&self, now: i64) -> bool {
    self.closed_at.is_none() && now >= self.due_at
}
```

### Error común

Meter el crecimiento diario **dentro** del tope:

```rust
// MAL — congela la deuda en el tope y reabre el agujero que ADR-004 §6 rechaza
min(cap_seconds, base_seconds + growth_per_day_seconds * days)
```

`ADR-004` §6 dice `min(tope, base + 1 min × abortos) + 5 min/día`: el tope acota la **base**, y lo que acota el crecimiento es la **caducidad**. Y si `days` no se clampea en `due_at`, una deuda vencida sigue creciendo mientras el daemon estuvo apagado.

### Referencias

- `docs/adr/ADR-004-clock-modes-and-penalties.md` §6 · `docs/adr/ADR-002-sqlite-schema.md` (Reglas de datos)

---

## Relación entre estos conceptos

Los tres son la misma idea que atraviesa el proyecto: **el estado se deriva, la escritura se desacopla y el esquema se versiona**. El hilo escritor evita que el disco dicte el ritmo de la sesión; el versionado evita que el esquema dicte cuándo se puede actualizar la app; y el cálculo evita que un contador persistido dicte cuánta deuda tenés. En los tres casos, se elimina una fuente de verdad que podría desincronizarse.

---

## Hallazgos de implementación

Cosas que aparecieron al codear y que no se ven venir leyendo el diseño.

### `strftime('%s', col)` devuelve TEXT

La conversión de vuelta de `TEXT datetime` a epoch seconds necesita cast explícito: `CAST(strftime('%s', col) AS INTEGER)`. Sin el `CAST`, `rusqlite` te da un `String` y el mapeo falla (o peor, si el tipo es laxo, pasa cualquier cosa). Con `NULL` o texto corrupto, el mapeo tiene que fallar **tipado** (`StoreError::Backend`), nunca devolver `0` en silencio.

### `wal_checkpoint(TRUNCATE)` no falla cuando no puede truncar

Devuelve una fila `(busy, log, checkpointed)`. Si un lector tiene una snapshot abierta, `busy != 0` y la truncación **no ocurrió** — pero la sentencia no es un error SQL. Hay que mirar el flag:

```rust
let busy: i64 = connection.query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |row| row.get(0))?;
if busy != 0 {
    return Err(backend(format!("wal_checkpoint(TRUNCATE) is incomplete (busy = {busy})")));
}
```

### Los ids asignados en el cliente necesitan una regla, no una convención

Como `save_*` devuelve el id antes de tocar el disco, el id lo asigna el adaptador con un contador por tabla. La trampa: un save con `id: Some(n)` explícito **también** tiene que avanzar el contador, o un `id: None` posterior reasigna un id vivo y choca contra la PK — con el error diferido a `flush()`, o sea invisible en el momento.

```rust
fn allocate_id(&mut self, kind: RecordKind, explicit: Option<i64>) -> i64 {
    let counter = &mut self.ids[kind.index()];
    match explicit {
        Some(id) => { *counter = (*counter).max(id); id }   // ← la línea que evita el bug
        None => { *counter += 1; *counter }
    }
}
```

### `flush()` no es `fsync`

Con `synchronous = NORMAL`, `flush()` garantiza que el hilo escritor aplicó todo lo encolado (commit en WAL), **no** que el dato sobreviva a un corte de energía. La doc del puerto lo dice explícitamente para que nadie lea más de lo que promete.

### Con dos conexiones, `:memory:` no existe

Una DB en memoria vive dentro de la conexión: el reader y el writer verían **dos bases distintas**. El diseño asume archivo; los tests usan temporales.

---

## Convención

- **Archivo**: `docs/learning/phase-02-store.md`
- **Título**: `# Fase 02 — Store SQLite`
- Cada fase crea su archivo al comenzar (un solo archivo por fase).
