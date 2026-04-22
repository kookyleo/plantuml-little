# PlantUML Test Fixtures

This directory contains `.puml` fixture files extracted from the PlantUML Java project's test suite.

## Source

- **Origin**: `/ext/plantuml/plantuml/src/test/java/` and `/ext/plantuml/plantuml/src/test/resources/`
- **Extraction**: manual + scripted from Java test classes

## Fixture Count by Category

| Category          | Count | Notes                                       |
|-------------------|------:|---------------------------------------------|
| sprite            |    40 | SVG sprite definitions and references        |
| preprocessor      |    38 | `!pragma`, `!include`, `!define`, etc.       |
| sequence          |    31 | Sequence diagrams + fragments + shapes       |
| misc              |    22 | Creole markup, skinparam, meta, links        |
| class             |    14 | Class / interface / generics                 |
| state             |    13 | State machines + pseudo-states + concurrent  |
| component         |    11 | Component, deployment, colors                |
| activity          |     8 | Activity v3 + swimlanes                      |
| wbs               |     5 | Work breakdown structure                     |
| erd               |     5 | Chen ER diagrams                             |
| bpm               |     4 | Business process model                       |
| object            |     4 | Object diagrams                              |
| activity_advanced |     3 | Advanced activity features                   |
| chart             |     3 | Chart diagrams                               |
| regex             |     3 | Regex visualisation                          |
| usecase           |     3 | Use case + boundaries                        |
| archimate         |     2 | Archimate diagrams                           |
| ebnf              |     2 | EBNF grammar diagrams                        |
| files_diagram     |     2 | Filesystem tree diagrams                     |
| flow              |     2 | Flowchart diagrams                           |
| git               |     2 | Git graph diagrams                           |
| packet            |     2 | Packet / protocol diagrams                   |
| timing            |     2 | Robust / concise timing                      |
| wire              |     2 | Wiring diagrams                              |
| board             |     1 | Board diagrams                               |
| chronology        |     1 | Chronology / timeline                        |
| creole            |     1 | Creole markup isolation test                 |
| def               |     1 | `@define` primitive test                     |
| ditaa             |     1 | ASCII art diagram                            |
| dot               |     1 | Graphviz DOT pass-through                    |
| gantt             |     1 | Gantt chart                                  |
| hcl               |     1 | HCL / tfstate diagram                        |
| jcckit            |     1 | JCCKit chart                                 |
| json              |     1 | JSON structure diagram                       |
| latex             |     1 | LaTeX diagram                                |
| math              |     1 | Math formula diagram                         |
| mindmap           |     1 | Mind map                                     |
| nwdiag            |     1 | Network diagram                              |
| pie               |     1 | Pie chart                                    |
| project           |     1 | Project plan                                 |
| salt              |     1 | Wireframe / UI mockup                        |
| sequence_puma     |     1 | Sequence (puma preset)                       |
| yaml              |     1 | YAML structure diagram                       |
| nonreg/           |    69 | Regression tests (simple 49, svg 8, xmi 5, scxml 5, graphml 2) |
| dev/              |    31 | Development tests (newline 17, jaws 12, newlinev2 2) |
| **Total**         |**342**|                                              |

## Directory Structure

```
fixtures/
├── activity/            # Activity v3 diagrams
├── activity_advanced/   # Advanced activity features
├── archimate/           # Archimate diagrams
├── board/               # Board diagrams
├── bpm/                 # Business process model
├── chart/               # Chart diagrams
├── chronology/          # Chronology / timeline
├── class/               # Class / interface / generics
├── creole/              # Creole markup isolation
├── def/                 # @define primitive
├── dev/                 # Development regression tests
├── ditaa/               # DITAA ASCII art
├── dot/                 # Graphviz DOT pass-through
├── ebnf/                # EBNF grammar diagrams
├── erd/                 # Entity-relationship (Chen notation)
├── files_diagram/       # Filesystem tree diagrams
├── flow/                # Flowchart diagrams
├── gantt/               # Gantt charts
├── git/                 # Git graph diagrams
├── hcl/                 # HCL / tfstate
├── jcckit/              # JCCKit chart
├── json/                # JSON visualisation
├── latex/               # LaTeX diagrams
├── math/                # Math formula diagrams
├── mindmap/             # Mind maps
├── misc/                # Creole, skinparam, metadata, hyperlinks
├── nonreg/              # Non-regression test suites
├── nwdiag/              # Network diagrams
├── object/              # Object diagrams
├── packet/              # Packet / protocol diagrams
├── pie/                 # Pie charts
├── preprocessor/        # Preprocessor directive tests
├── project/             # Project plan
├── regex/               # Regex visualisation
├── salt/                # Salt / wireframe UI
├── sequence/            # Sequence diagrams
├── sequence_puma/       # Sequence (puma preset)
├── sprite/              # SVG sprite definitions
├── state/               # State machine diagrams
├── timing/              # Timing diagrams
├── usecase/             # Use case diagrams
├── wbs/                 # Work breakdown structure
├── wire/                # Wiring diagrams
└── yaml/                # YAML visualisation
```
