# Documentación de TPT

Mapa de navegación de la documentación del proyecto. Este es el **índice único** — los agentes IA y las personas lo usan para ubicarse.

## Docs

| Área | Documento | Contenido |
|------|-----------|-----------|
| **Estado actual** (mutable) | `docs/handoff.md` | Fase activa, próximo paso, riesgos |
| Visión | `docs/vision.md` | Alcance, funcionalidades core, decisiones D1–D20 |
| Plan de fases POR FEATURE | `docs/phases.md` | Fases con scope, criterios de salida, pasos TDD |
| Aprendizaje | `docs/learning/` | Conceptos por fase (`phase-NN-name.md`) |
| Progreso (log) | `docs/progress-log/` | Historial por fase (`phase-NN-name.md`) |
| Decisiones | `docs/adr/` | ADRs 001–008 (`ADR-0NN-*.md`) |
| Arquitectura | `docs/architecture.md` | Crates, puertos, IPC, modelo de datos, testing, CI |

## Fuera de docs

| Documento | Contenido |
|-----------|-----------|
| `README.md` | Portafolio público (inglés) |
| `AGENTS.md` | Contexto estático para agentes IA |
| `config.toml` | Configuración de ejemplo/por defecto |

## Reglas de docs

- Idioma: español. Nombres de carpetas/archivos en inglés.
- Formato: Markdown.
- Mantener `handoff.md` actualizado al iniciar/cerrar sesión.
- Cada fase crea `learning/phase-NN-name.md` y `progress-log/phase-NN-name.md` al comenzar (justo-a-tiempo).
- ADRs: una decisión = un archivo en `docs/adr/`. El detalle vive ahí; no se duplica acá.
- **Sin diagramas por defecto.** Solo si algo es imposible de explicar con texto.
- **No duplicar**: si un párrafo se repite en dos archivos, está mal ubicado.
