# PlantUML Test Fixtures

This directory contains `.puml` fixture files extracted from the PlantUML Java project's test suite.

## Source

- **Origin**: `/ext/plantuml/plantuml/src/test/java/` and `/ext/plantuml/plantuml/src/test/resources/`
- **Extraction**: manual + scripted from Java test classes

## Fixture Count by Category

| Category     | Count | Notes                                       |
|--------------|------:|---------------------------------------------|
| sprite       |    40 | SVG sprite definitions and references        |
| preprocessor |    38 | `!pragma`, `!include`, `!define`, etc.       |
| sequence     |    31 | Sequence diagrams + fragments + shapes       |
| misc         |    18 | Creole markup, skinparam, meta, links        |
| class        |    14 | Class / interface / generics / object        |
| state        |    13 | State machines + pseudo-states + concurrent  |
| component    |    10 | Component, deployment, colors                |
| activity     |     8 | Activity v3 + swimlanes                      |
| wbs          |     5 | Work breakdown structure                     |
| erd          |     5 | Chen ER diagrams                             |
| usecase      |     3 | Use case + boundaries                        |
| timing       |     2 | Robust / concise timing                      |
| yaml         |     1 | YAML structure diagram                       |
| salt         |     1 | Wireframe / UI mockup                        |
| nwdiag       |     1 | Network diagram                              |
| mindmap      |     1 | Mind map                                     |
| json         |     1 | JSON structure diagram                       |
| gantt        |     1 | Gantt chart                                  |
| dot          |     1 | Graphviz DOT pass-through                    |
| ditaa        |     1 | ASCII art diagram                            |
| nonreg/      |    64 | Regression tests (simple 49, svg 8, scxml 5, graphml 2) |
| dev/         |    31 | Development tests (jaws 12, newline 17, v2 2)|
| **Total**    |**290**|                                              |

## Directory Structure

```
fixtures/
├── activity/       # Activity v3 diagrams
├── class/          # Class / interface / object diagrams
├── component/      # Component / deployment diagrams
├── dev/            # Development regression tests
├── ditaa/          # DITAA ASCII art
├── dot/            # Graphviz DOT pass-through
├── erd/            # Entity-relationship (Chen notation)
├── gantt/          # Gantt charts
├── json/           # JSON visualization
├── mindmap/        # Mind maps
├── misc/           # Creole, skinparam, metadata, hyperlinks
├── nonreg/         # Non-regression test suites
├── nwdiag/         # Network diagrams
├── object/         # (merged with class/)
├── preprocessor/   # Preprocessor directive tests
├── salt/           # Salt / wireframe UI
├── sequence/       # Sequence diagrams
├── sprite/         # SVG sprite definitions
├── state/          # State machine diagrams
├── timing/         # Timing diagrams
├── usecase/        # Use case diagrams
├── wbs/            # Work breakdown structure
└── yaml/           # YAML visualization
```
