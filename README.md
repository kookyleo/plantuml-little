# plantuml-little

A lightweight Rust implementation of [PlantUML](https://plantuml.com/), focused on converting `.puml` files to SVG output.

## Goals

- **Input**: `.puml` files (PlantUML text)
- **Output**: `.svg` files only
- **Form**: Library (`lib`) + CLI binary
- **Layout**: [Graphviz](https://graphviz.org/) via vizoxide (for Class / State / Component / ERD / UseCase) + built-in engine (for Sequence / Activity / Timing / Gantt / JSON / Mindmap / WBS / Salt / DITAA / NWDiag)

## Supported Diagram Types (17)

Class, Sequence, Activity v3, State, Component/Deployment, Use Case, Object,
Timing, ERD (Chen), Gantt, JSON, YAML, Mindmap, WBS, DITAA, NWDiag, Salt/Wireframe,
DOT (Graphviz pass-through)

## Features

- Full preprocessor: variables, functions, conditionals, loops, includes, themes, 35+ builtins
- Skinparam style system with rose default theme
- Creole rich text markup (bold / italic / links / lists / tables / colors / fonts)
- SVG sprite inline embedding
- Sequence combined fragments, state pseudo-states, activity swimlanes
- CJK / Unicode text width support
- Error reporting with line/column tracking

See [FEATURES.md](FEATURES.md) for the complete support matrix.

## Usage

```bash
# CLI
plantuml-little input.puml -o output.svg

# Library
let svg = plantuml_little::convert(puml_source)?;
```

## Prerequisites

- Rust 1.70+
- Graphviz (`apt install graphviz` / `brew install graphviz`)

## Non-Goals

- GUI, web server, FTP, pipe mode
- Output formats other than SVG
- PlantUML Server URL encoding/decoding
- Security sandbox system

## Test Coverage

1,319 unit tests + 183 integration tests = **1,502 tests**, 296 fixture files, 0 warnings.

## Acknowledgments

This project is an independent Rust reimplementation of [PlantUML](https://plantuml.com/), created by Arnaud Roques. We deeply appreciate the PlantUML team's work in making diagram-as-code accessible to everyone. This project fully adopts the same licensing scheme as upstream PlantUML.

We periodically track upstream updates. All specification-level behavior follows the upstream standard. Issues and pull requests are welcome.

## License

PlantUML is licensed under several open-source licenses. As a reimplementation following the same language specification, this project adopts the same multi-license approach — you can choose the one that suits you best:

- [GPL license](https://www.gnu.org/licenses/gpl-3.0.html)
- [LGPL license](https://www.gnu.org/licenses/lgpl-3.0.html)
- [Apache license](https://www.apache.org/licenses/LICENSE-2.0)
- [Eclipse Public license](https://www.eclipse.org/legal/epl-2.0/)
- [MIT license](https://opensource.org/licenses/MIT)

For more information on the upstream licensing, see the [PlantUML license FAQ](https://plantuml.com/en/faq#ddbc9d04378ee462).
