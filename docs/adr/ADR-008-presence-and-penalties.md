# ADR-008: Presencia — Idle, Pantalla Apagada y Suspensión

- **Estado**: Aceptado
- **Fecha**: 2026-09-17 · *(revisado: quickshell reemplaza a `hypridle`)*
- **Reemplaza**: `IdleSource` del diseño anterior

## Contexto

El contrato necesita saber **si el usuario estuvo presente**. Eso implica tres cosas distintas: inactividad de teclado, pantalla apagada y suspensión/apagado. En Wayland una aplicación cualquiera **no puede** consultar su idle time, y el proyecto exige **cero runtimes async y mínimo consumo de memoria**.

**El entorno real importa:** esta sesión corre **`caelestia-shell`** (quickshell). Su `modules/IdleMonitors.qml` crea un `IdleMonitor` — protocolo `ext-idle-notify-v1` — por cada entrada de `general.idle.timeouts` en `~/.config/caelestia/shell.json`. `hypridle` está instalado pero **sin configurar y sin correr**, y `hyprlock` tampoco se usa: el lock es propio del shell.

## Decisión

### 1. Hooks de `caelestia-shell` (fuente primaria)

El shell ya resolvió la parte Wayland. Consumimos sus eventos agregando entradas propias en `general.idle.timeouts`. Las acciones en **array** no coinciden con ninguna acción conocida y caen a `Quickshell.execDetached`:

```json
{
  "timeout": 420,
  "idleAction":   ["tpt-cli", "presence", "--state", "idle"],
  "returnAction": ["tpt-cli", "presence", "--state", "active"],
  "inhibitWhenAudio": false
}
```

`tpt-cli presence` habla con el daemon por UDS. **No abre Wayland, no habla D-Bus, no linkea nada.**

### 2. Salto de reloj (backstop, siempre activo)

`CLOCK_MONOTONIC` **no avanza durante la suspensión**; el reloj de pared sí:

```
salto = (wall_ahora − wall_antes) − (mono_ahora − mono_antes)
```

Cubre **suspensión, apagado y reinicio** con verdad del kernel, sin APIs del SO.

### 3. Qué señales importan

| Señal | Fuente | Efecto en el contrato |
|-------|--------|----------------------|
| Idle de teclado | hook `presence --state idle` | Gap si dura ≥ `min_gap_seconds` |
| **Pantalla apagada** (`dpms off`) | hook `presence --state screen-off` | `L1` acumula (gracia 0) · `L2` **aborta inmediato** |
| Suspensión / apagado | salto de reloj | Gap no acreditado |
| **Bloqueo de pantalla** | — | **Ignorado.** No es señal del contrato |

**Por qué el lock se ignora:** con el lock a los 5 min, tratarlo como "se fue" mataría cualquier pausa de lectura o de pensar. El **screen-off** (12 min de inactividad) sí implica ausencia prolongada, y es el único que dispara aborto inmediato en `L2`. El lock queda para lo que es: seguridad de la sesión.

### 4. El gate de `inhibitWhenAudio`

`IdleMonitors.qml` evalúa **primero** un gate global: si `GlobalConfig.general.idle.inhibitWhenAudio` es `true`, **ningún** monitor corre mientras suena audio. Eso apagaría también el hook de TPT.

**Solución:** el global va en `false`, y cada entrada de lock/dpms/suspend lleva `inhibitWhenAudio: true`. Resultado: un video no te bloquea la pantalla ni te suspende la máquina, pero **TPT sí sigue viendo la presencia real**.

### Por qué NO las alternativas

| Alternativa | Por qué se descarta |
|-------------|---------------------|
| **`hypridle`** | No está configurado ni corriendo; el idle lo posee quickshell. Escribir un `hypridle.conf` sería levantar un segundo gestor de idle en paralelo |
| **logind `IdleHint` / `LockedHint` (D-Bus)** | `CanIdle=yes` / `CanLock=yes`, pero **nadie los setea**: quickshell no habla D-Bus. Verificado con `loginctl show-session`. Leerlos sería código muerto |
| **Cliente Wayland propio (`ext-idle-notify-v1`)** | Duplica el trabajo del shell y acopla el daemon al compositor |
| **Leer `/dev/input/*`** | Requiere privilegios de root |

## Consecuencias

- **Cero dependencias nuevas**: sin `zbus`, sin `dbus`, sin `wayland-client`.
- **Requisito:** el `caelestia-shell` corriendo y con la entrada de TPT en `shell.json`. Es el equivalente actual del viejo "hypridle debe estar prendido".
- **Degradación elegante:** sin shell, la app sigue funcionando con el salto de reloj; solo se pierden los avisos de idle y la detección de screen-off.
- El dominio solo conoce `PresenceSource`; los adaptadores son reemplazables y fakeables.
- **Heartbeat:** en `L1`/`L2` no se puede arrancar sin señal fresca de presencia — si el shell no reporta, el contrato no tiene juez.

## Referencias

- `docs/architecture.md` (Presencia) · `ADR-003` (puertos) · `ADR-004` (niveles y penalizaciones) · `ADR-007`
