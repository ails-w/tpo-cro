# ADR-009: Branching y Resincronización de `dev`

- **Estado**: Aceptado
- **Fecha**: 2026-09-23
- **Reemplaza**: `ADR-006-branching-and-protection.md`

## Contexto

`ADR-006` fijó dos ramas vivas (`dev` de trabajo, `main` estable) con **historial lineal** en ambas, **rebase** como método de merge de `dev → main`, y **sin force-push en `dev`**.

Esa combinación es autocontradictoria:

1. Mergear `dev → main` con **rebase** reescribe los commits en `main` (hashes nuevos).
2. `dev` queda con los hashes viejos: **divergente**, aunque el contenido sea idéntico.
3. Para resincronizarla hacía falta una de tres cosas, y las tres estaban prohibidas: force-push (`non_fast_forward`), mergear `main` dentro de `dev` (`required_linear_history`), o borrarla y recrearla (`deletion`).

Resultado: **cualquier PR `dev → main` dejaba `dev` huérfana para siempre**. Pasó con la Fase 1 y volvió a bloquear el push de la Fase 2, que terminó subiendo el trabajo en una rama `feat/*` descartable — exactamente la fricción que este ADR elimina.

## Decisión

Se mantiene el modelo de **dos ramas** (`dev` de trabajo, `main` estable) y se corrige la regla que lo hacía insostenible.

| Rama | Reglas del ruleset |
|------|--------------------|
| `main` | sin borrado · sin force-push · historial lineal · **PR obligatorio** (0 aprobaciones, solo-dev) · **check requerido** `fmt + clippy + test` |
| `dev` | sin borrado · historial lineal · **force-push permitido** |

`dev` pierde `non_fast_forward` **a propósito**: es la única regla que permite devolverla al estado de `main` después de cada fase.

**Sin ramas `feat/*` por defecto.** `dev` ya es la rama de integración; una rama corta por fase no aporta aislamiento y sí una PR y un merge extra.

### Ciclo de una fase

1. **Trabajar en `dev`** con pushes directos (sin PR, sin ramas cortas).
2. Al cerrar la fase: **PR `dev → main`**, merge con **rebase** (preserva el commit por feature; el squash aplanó la historia una vez y se descartó).
3. **Resincronizar `dev` con `main`** — el paso que faltaba:

   ```bash
   git checkout dev
   git fetch origin
   git reset --hard origin/main
   git push --force-with-lease origin dev
   ```

Sin el paso 3, `dev` queda divergente y el próximo push directo es rechazado por `non_fast_forward`… que ya no existe, pero el push igual sería un force-push encubierto. El paso es explícito para que no se olvide.

### Por qué no las alternativas

- **Permitir merge commits en `dev`** (quitar `required_linear_history`) y resincronizar mergeando `main` en `dev`: deja merge commits que el PR a `main` después aplana con rebase. Historial sucio sin beneficio.
- **Squash en `dev → main`**: aplanaba los commits por feature. Rechazado explícitamente; ya rompió la historia una vez.
- **`feat/*` por fase**: funciona, pero agrega una rama descartable por fase sin aportar aislamiento real.

## Consecuencias

- `dev` vuelve a ser resincronizable: el ciclo de fase tiene 3 pasos explícitos.
- El force-push en `dev` es una operación **acotada y esperada** (`--force-with-lease`, un solo autor). `main` sigue sin aceptarlo nunca.
- `main` conserva la garantía que importa: no recibe pushes directos y todo entra por PR con checks verdes.
- ⚠️ **Si el proyecto suma un segundo desarrollador**, hay que revisar este ADR: `dev` compartida con force-push permitido es una combinación riesgosa.

## Gotcha operativo: scope `workflow`

Sin cambios respecto de `ADR-006`: un token **OAuth App** (el de `gh`) no puede crear ni actualizar archivos en `.github/workflows/` sin el scope `workflow`, y aplica también al merge de un PR que toque workflows. Con SSH no aplica; con `gh` sobre HTTPS hace falta un PAT con permiso **Workflows: Read and write**.

## Referencias

- `ADR-006-branching-and-protection.md` (reemplazado) · `AGENTS.md` (commits, DoD y flujo de ramas) · `docs/architecture.md` (CI)
