# Fase 01 — Dominio, configuración y puertos

> Conceptos de la fase 1 del proyecto.
> Log de la fase → `docs/progress-log/phase-01-domain.md`

---

## Contrato aditivo como invariante de dominio

### Qué es

Un contrato de tiempo donde la única operación permitida sobre la meta es **crecer**. Se puede `+N` minutos; no existe `-N`. Tampoco existe pausa en Pomodoro ni en Focus.

### Qué problema resuelve

Un timer ajustable es un timer negociable: cuando el compromiso aprieta, aparece la tentación de bajarle dos minutos. Si la meta solo puede crecer, no hay negociación posible: o la cumplís, o la cancelás pagando un costo.

### Para qué sirve en este proyecto

Es la identidad del producto (`docs/vision.md` D7). Todo el motor de sesiones se diseña alrededor de esta restricción: el tipo que modela la meta no expone una operación de resta, así que **el estado inválido no es representable**. No es una validación — es una imposibilidad de tipo.

### Cómo se usa

```rust
pub struct SessionTarget {
    seconds: u32,
}

impl SessionTarget {
    /// Única mutación permitida: extender.
    pub fn extend(&mut self, seconds: u32) { self.seconds += seconds; }
    pub fn remaining(&self, credited: u32) -> u32 { self.seconds.saturating_sub(credited) }
    // No existe `shrink()` ni `pause()`: el contrato no lo permite.
}
```

### Error común

Modelar la meta como un `u32` mutable en la sesión y "validar" en el handler que no se reduzca. La validación se olvida o se saltea; el tipo no. La regla se hace cumplir por construcción, no por disciplina.

### Referencias

- `docs/adr/ADR-004-clock-modes-and-penalties.md` · `docs/vision.md` (D7, D8)

---

## Presencia sin dependencias: hooks de `hypridle` + salto de reloj

### Qué es

Un puerto (`PresenceSource`) que responde una sola pregunta — *¿el usuario estuvo presente?* — con dos adaptadores:

- **`HypridleHookSource`**: `hypridle` ejecuta `tpt-cli presence --state <s>` en sus `on-timeout` / `on-resume` / `on_lock_cmd` / `on_unlock_cmd`.
- **`ClockGapSource`**: compara el salto del reloj de pared contra el monotónico.

### Qué problema resuelve

En Wayland, una app no puede consultar su idle time: el compositor lo sabe y no lo publica sin suscripción a un protocolo. Y el proyecto prohíbe runtimes async y dependencias pesadas.

### Para qué sirve en este proyecto

Es lo que permite detectar idle, bloqueo de pantalla y suspensión **sin** D-Bus ni libwayland. `hypridle` ya es el cliente Wayland que consume `ext-idle-notify-v1`: en vez de duplicar ese trabajo, consumimos sus hooks.

El backstop del reloj cubre lo que `hypridle` no puede: `CLOCK_MONOTONIC` no avanza durante la suspensión, así que `(Δwall − Δmono)` grande significa que la máquina estuvo apagada o dormida. Es verdad del kernel, sin APIs del SO.

### Cómo se usa

```ini
# ~/.config/hypr/hypridle.conf
listener { timeout = 420 ; on-timeout = tpt-cli presence --state idle --seconds 420
                          ; on-resume  = tpt-cli presence --state active }
```

### Error común

Ir directo a **logind por D-Bus** asumiendo que `IdleHint` funciona. **Hyprland no implementa D-Bus**: nunca llama `SetIdleHint()`, así que esa propiedad queda muerta. Se paga una dependencia (y el runtime async que trae `zbus`) para leer un valor que nunca cambia.

### Referencias

- `docs/adr/ADR-008-presence-and-penalties.md` · `docs/architecture.md` (Presencia)

---

## Puertos vs adaptadores en un dominio sin SO

### Qué es

El dominio (contrato, penalizaciones, métricas) declara **traits** para todo lo externo: reloj, presencia, persistencia, configuración, notificaciones, transporte. Los adaptadores concretos viven en `tpt-daemon`.

### Qué problema resuelve

Sin esa frontera, testear el contrato exige Hyprland, una sesión gráfica y disco. Con ella, el motor se testea con fakes en milisegundos.

### Para qué sirve en este proyecto

Permite que las reglas más delicadas —contrato aditivo, escalada de inactividad, penalizaciones— se prueben en CI sin compositor. También hace reemplazable el adaptador de presencia si algún día cambia el entorno.

### Cómo se usa

```rust
#[cfg(test)]
mod tests {
    // El dominio no sabe que existe Hyprland.
    let clock = FakeClock::new();
    let presence = FakePresence::default();
    let engine = SessionEngine::new(Box::new(clock), Box::new(presence));
}
```

### Error común

Dejar que el dominio importe un tipo del adaptador ("solo para el `PathBuf`" o "solo para el error"). En cuanto el core conoce un tipo del daemon, la frontera se rompió y los tests empiezan a necesitar el entorno real.

### Referencias

- `docs/adr/ADR-003-hexagonal-ports.md` · `docs/architecture.md` (Puertos)

---

## Relación entre estos conceptos

Los tres son la misma idea aplicada en tres niveles: **imponer la regla por diseño, no por disciplina**. El contrato aditivo hace irrepresentable acortar la meta; la presencia sin dependencias evita cargar el daemon con lo que otro proceso ya resuelve; y la frontera de puertos hace que el dominio no pueda acoplarse accidentalmente al SO. En los tres casos, el error deja de ser posible en vez de ser detectado.

---

## Convención

- **Archivo**: `docs/learning/phase-01-domain.md`
- **Título**: `# Fase 01 — Dominio, configuración y puertos`
- Cada fase crea su archivo al comenzar (un solo archivo por fase).
