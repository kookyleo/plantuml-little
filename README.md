# plantuml-little

[中文](README.zh.md) | English

A lightweight Rust reimplementation of [PlantUML](https://plantuml.com/), targeting byte-exact SVG output parity with Java PlantUML **v1.2026.2**.

## What Is This

plantuml-little takes `.puml` source text and produces `.svg` output — the same as Java PlantUML, but as a native Rust library + CLI with zero JVM dependency. The goal is **identical SVG output** for all supported diagram types, verified by 337 byte-exact reference tests against the upstream Java release.

## Alignment Status

| | |
|---|---|
| **Upstream version** | PlantUML v1.2026.2 (`bb8550d`) |
| **Reference tests** | 337 passed / 0 failed / 3 ignored |
| **Unit tests** | 2,693 |
| **Integration tests** | 185 |
| **Total tests** | **3,215** |

## Supported Diagram Types (29)

All types below produce SVG output byte-exactly matching Java PlantUML v1.2026.2.

| Type | Start Tag | Layout Engine |
|------|-----------|---------------|
| Class | `@startuml` | Graphviz (Smetana) |
| Sequence | `@startuml` | Built-in (Puma / Teoz) |
| Activity v3 | `@startuml` | Built-in |
| State | `@startuml` | Graphviz |
| Component / Deployment | `@startuml` | Graphviz |
| Use Case | `@startuml` | Graphviz |
| Object | `@startuml` | Graphviz |
| Timing | `@startuml` | Built-in |
| ERD (Chen) | `@startchen` | Graphviz |
| Gantt | `@startgantt` | Built-in |
| JSON | `@startjson` | Built-in |
| YAML | `@startyaml` | Built-in |
| Mindmap | `@startmindmap` | Built-in |
| WBS | `@startwbs` | Built-in |
| NWDiag | `@startnwdiag` | Built-in |
| Salt / Wireframe | `@startsalt` | Built-in |
| DOT | `@startdot` | Graphviz pass-through |
| EBNF | `@startebnf` | Built-in |
| Regex | `@startregex` | Built-in |
| BPM | `@startbpm` | Built-in |
| Board | `@startboard` | Built-in |
| Chronology | `@startchronology` | Built-in |
| Chart | `@startchart` | Built-in |
| Pie | `@startpie` | Built-in |
| HCL | `@starthcl` | Built-in |
| Flow | `@startflow` | Built-in |
| Wire | `@startwire` | Built-in |
| Archimate | `@startuml` | Graphviz |
| Packet | `@startpacket` | Built-in |

### Additional Types (text / passthrough)

| Type | Notes |
|------|-------|
| Creole | `@startcreole` — rich text markup rendering |
| Def | `@startdef` — plain text display |
| Math / LaTeX | `@startmath` / `@startlatex` — formula placeholder (Java requires external tools) |
| Git | `@startgit` — git log visualization |
| Files | `@startfiles` — file tree display |

### Intentionally Unsupported

| Type | Reason |
|------|--------|
| DITAA | Java delegates to a third-party rasterizer (no SVG mode). Implementing ASCII art → SVG from scratch is out of scope. |
| JCCKIT | Java AWT charting library, renders to `Graphics2D` only. No Rust equivalent. |
| Project (Gantt v2) | Java stable v1.2026.2 itself does not render this type. |

## Features

- **Full preprocessor**: variables, functions, conditionals, loops, includes, themes, 35+ built-in functions
- **Skinparam style system** with rose default theme
- **Creole rich text**: bold / italic / underline / strike / colors / fonts / links / tables / lists
- **SVG sprite embedding** with viewBox-aware scaling
- **OpenIconic icons** (`<&icon>` syntax, 223 embedded icons)
- **Handwritten mode** (`skinparam handwritten true`)
- **Gradient fills** (linear / radial)
- **Sequence features**: 8 participant shapes, 8+ combined fragments, dividers, autonumber
- **Activity features**: swimlanes, goto/label, break, backward loops
- **State features**: fork/join, choice, history, concurrent regions
- **CJK / Unicode** character width support
- **Error reporting** with line/column tracking

See [FEATURES.md](FEATURES.md) for the complete support matrix.

## Usage

```bash
# CLI
plantuml-little input.puml -o output.svg

# Library
let svg = plantuml_little::convert(puml_source)?;
```

## Prerequisites

- Rust 1.82+
- Graphviz (`apt install graphviz` / `brew install graphviz`)

## Non-Goals

- GUI, web server, FTP, pipe mode
- Output formats other than SVG (no PNG / PDF / EPS / ASCII)
- PlantUML Server URL encoding/decoding
- ELK layout engine
- Security sandbox

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
