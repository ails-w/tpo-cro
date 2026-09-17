# ADR-006: Branching y Protección de Ramas

- **Estado**: Aceptado
- **Fecha**: 2026-09-14

## Contexto

El proyecto tiene un solo desarrollador, corre en un repo público (`github.com:ails-w/tpo-cro`) y necesita un flujo de revisión que no agregue fricción innecesaria, pero que impida romper la rama estable. El flujo anterior ("solo existe `main`") no contemplaba integración ni checks obligatorios.

## Decisión

Dos ramas vivas:

```
feat/* · fix/* · chore/* · docs/*   (cortas)
        │  PR + checks
        ▼
       dev        ← integración · push directo permitido
        │  PR + checks
        ▼
      main        ← estable · default branch
```

### Protección (Rulesets de GitHub)

| Rama | Reglas |
|------|--------|
| `main` | Sin borrado · sin force-push · historial lineal · **PR obligatorio** (0 aprobaciones, solo-dev) · **check requerido** `fmt + clippy + test` |
| `dev` | Sin borrado · sin force-push · historial lineal. **Sin `required_status_checks`** |

**Por qué `dev` no lleva `required_status_checks`:** en rulesets esa regla bloquea la *actualización del ref* (obliga a pasar por otra rama), lo que mataría el push directo a `dev`. En `dev` la protección es contra force-push y borrado, nada más.

**Historial lineal** obliga a mergear con **rebase o squash**. Se usa **rebase** para preservar el commit por feature.

### Qué NO se delega a la tooling

La revisión del trabajo la hace el humano (o un agente cuando se le pide). Los checks verifican compile/lint/test, no diseño.

## Alternativas consideradas

- **Solo `main` con PRs**: cada cambio de una línea requiere PR; fricción alta. Descartada.
- **GitFlow completo (`develop`, `release/*`, `hotfix/*`)**: sobredimensionado para un solo desarrollador. Descartada.
- **`required_status_checks` en `dev`**: rompe el push directo, que es justamente lo que se quiere ahí. Descartada.

## Consecuencias

- `main` nunca recibe un push directo: todo entra por PR con checks verdes.
- `dev` es el lugar de trabajo diario, con red de seguridad contra force-push.
- Se puede mergear la PR propia (0 aprobaciones) porque el proyecto es de un solo autor.

## Gotcha operativo: scope `workflow`

Un token **OAuth App** (el de `gh`) **no puede** crear ni actualizar archivos en `.github/workflows/` sin el scope `workflow`, y esto aplica **también al merge** de un PR que toca workflows.

- Si se pushea con **SSH** (llave de usuario), la restricción no aplica.
- Si se trabaja con `gh` sobre HTTPS, se necesita un PAT con permiso **Workflows: Read and write**.

## Referencias

- `AGENTS.md` (commits, DoD y flujo de ramas) · `docs/architecture.md` (CI)
