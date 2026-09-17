# ADR-008: Presencia — Idle, Bloqueo de Pantalla y Suspensión

- **Estado**: Aceptado
- **Fecha**: 2026-09-14
- **Reemplaza**: `IdleSource` del diseño anterior

## Contexto

El contrato de tiempo necesita saber **si el usuario estuvo presente**. Eso implica detectar tres cosas distintas:

1. **Inactividad de teclado/mouse** (¿está pensando o se fue?).
2. **Bloqueo de pantalla** (se fue, sin ambigüedad).
3. **Suspensión / apagado** (la máquina no estaba disponible).

En Wayland, una aplicación cualquiera **no puede** conocer su idle time: el compositor lo sabe, pero no lo publica sin suscripción a un protocolo. Y el proyecto exige **cero runtimes async y mínimo consumo de memoria** (`AGENTS.md`).

## Decisión

**Dos fuentes, ninguna dependencia nueva:**

### 1. Hooks de `hypridle` (fuente primaria)

`hypridle` ya es un cliente Wayland que consume `ext-idle-notify-v1`. No duplicamos ese trabajo: consumimos sus eventos.

```ini
# ~/.config/hypr/hypridle.conf
general {
    on_lock_cmd   = tpt-cli presence --state locked
    on_unlock_cmd = tpt-cli presence --state unlocked
}

listener { timeout = 180 ; on-timeout = tpt-cli presence --state idle --seconds 180 }
listener { timeout = 300 ; on-timeout = tpt-cli presence --state idle --seconds 300 }
listener { timeout = 420 ; on-timeout = tpt-cli presence --state idle --seconds 420 ; on-resume = tpt-cli presence --state active }
```

`tpt-cli presence` habla con el daemon por UDS. **No abre Wayland, no habla D-Bus, no linkea nada.**

### 2. Salto de reloj (backstop, siempre activo)

`CLOCK_MONOTONIC` **no avanza durante la suspensión**; el reloj de pared sí. Por lo tanto:

```
salto = (wall_ahora − wall_antes) − (mono_ahora − mono_antes)
```

Si el salto es grande, la máquina estuvo suspendida o apagada. Cubre **suspensión, apagado y reinicio** con verdad del kernel, sin APIs del SO.

### Por qué NO se usan las otras alternativas

| Alternativa | Por qué se descarta |
|-------------|---------------------|
| **logind `IdleHint` (D-Bus)** | **Hyprland no implementa D-Bus**: nunca llama `SetIdleHint()`, así que la propiedad queda muerta. Leerla sería código muerto. |
| **logind `PrepareForSleep`/`PrepareForShutdown`** | Confiables, pero obligan a sumar `zbus` (trae maquinaria async) o `dbus` (linkea librería C). El salto de reloj cubre lo mismo sin dependencias. |
| **Cliente Wayland propio (`ext-idle-notify-v1`)** | Duplica el trabajo de `hypridle` y acopla el daemon al compositor. |
| **Leer `/dev/input/*`** | Requiere privilegios de root. Inaceptable. |

## Escalada y efecto en el crédito

| Situación | Aviso | Descuento de crédito |
|-----------|-------|----------------------|
| < 3 min sin input | — | ninguno (gracia) |
| 3 min | informativo | ninguno |
| 5 min | fuerte | ninguno |
| ≥ 7 min | fuerte | `max(0, gap − 180 s)` |
| Bloqueo de pantalla | — | el gap completo |
| Suspensión / apagado | — | el gap completo |

**Los avisos nunca descuentan tiempo.** El descuento lo produce la ausencia real, no la notificación. Los niveles de estrictez y sus penalizaciones → `ADR-004`.

## Consecuencias

- **Cero dependencias nuevas**: sin `zbus`, sin `dbus`, sin `wayland-client`.
- **Degradación elegante**: si `hypridle` no está corriendo, la app funciona igual con el salto de reloj; solo se pierden los avisos de idle y la detección de lock.
- El dominio solo conoce `PresenceSource`; los adaptadores son reemplazables y fakeables.
- Costo: la configuración de `hypridle` es responsabilidad del usuario (se documenta en el README y en `config.toml`).

## Referencias

- `docs/architecture.md` (Presencia) · `ADR-003` (puertos) · `ADR-004` (penalizaciones) · `ADR-007`
