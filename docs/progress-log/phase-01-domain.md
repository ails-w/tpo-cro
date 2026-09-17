# Fase 1: Pivote documental + dominio, config y puertos — Log

> Log HISTÓRICO de la fase. Se acumula, no se borra.
> Estado actual → `docs/handoff.md` · Conceptos → `docs/learning/phase-01-domain.md`

## Estado

**Estado**: En Progreso
**Última Actualización**: 2026-09-14

## Objetivos

- Alinear toda la documentación con el pivote *timer-first*.
- Dejar en `tpt-core` los modelos del dominio, la validación de config y los puertos.

## Progreso

- [x] Feature 1.0 — Pivote documental (2026-09-14)
- [ ] Feature 1.1 — Configuración TOML
- [ ] Feature 1.2 — Validación de presets y niveles
- [ ] Feature 1.3 — Actividades, proyectos y agenda
- [ ] Feature 1.4 — Sesiones y entradas de tiempo
- [ ] Feature 1.5 — Puertos

## Tareas Completadas

### 2026-09-14 — Feature 1.0: Pivote documental

- **Descripción**: Reescritura completa de la documentación para el alcance *timer-first*. Se abandonó el tracking pasivo de ventanas.
- **Reescritos**: `docs/vision.md`, `docs/phases.md`, `docs/architecture.md`, `docs/index.md`, `docs/handoff.md`, `README.md`, `AGENTS.md`, `config.toml`.
- **ADRs reestructurados**: 001 (arquitectura), 002 (esquema SQLite), 003 (puertos), 004 (contrato y penalizaciones). **Nuevos**: 005 (tpt-cli), 006 (branching), 007 (pivote), 008 (presencia).
- **Documentos eliminados**: ADRs de blocklist, retención/BTRFS y capas de tracking; `docs/development-plan.md` (fusionado); `docs/diagrams/`, `docs/learning/` y `docs/progress-log/` READMEs (duplicaban índices).
- **Tests**: N/A (fase de documentación).

## Decisiones

1. **Sin tracking de apps** — el compromiso pasa a ser autoimpuesto y el timer se vuelve el centro. Registrado en `ADR-007`.
2. **Presencia vía `hypridle` + salto de reloj** — cero dependencias nuevas; se descarta D-Bus porque Hyprland nunca llama `SetIdleHint` (`ADR-008`).
3. **Contrato aditivo** — una sesión solo puede extenderse, nunca acortarse (`ADR-004`).
4. **SQLite sin tiers** — sin flood de eventos, las métricas se calculan on-the-fly (`ADR-002`).
5. **Contrato de compromiso en la DB con checksum**, no en `config.toml`, que es editable a mano (`ADR-004`).
6. **Renumeración de ADRs** en vez de acumular ADRs superados, por ser un proyecto pre-código en reestructuración.

## Problemas

1. **`dev` quedó vacía de Fase 0** — se creó desde `main` y los commits de Fase 0 viven en la rama remota `chore/phase-0-setup`. Pendiente llevarlos a `dev`.
2. **ADRs del diseño viejo dispersos** — se resolvió renumerando, en lugar de cargar el repo con ADRs "Reemplazado por…" para decisiones que nunca llegaron a implementarse.

## Pendientes

- 🔴 **Definir niveles de estrictez y default** (único tema abierto). Propuesta en `ADR-004` §5.
- Features 1.1 a 1.5 (dominio, config y puertos).
