# plantuml-little Feature Support

Aligned with Java PlantUML **v1.2026.2** (`bb8550d`).

## Diagram Types — 29 fully implemented + 5 text/passthrough + 3 unsupported

### Fully Implemented (byte-exact SVG parity with Java)

| Type | Start Tag | Layout Engine | Ref Tests |
|------|-----------|---------------|-----------|
| Class | `@startuml` | Graphviz (Smetana) | 14 |
| Sequence | `@startuml` | Built-in (Puma / Teoz) | 32 |
| Activity v3 | `@startuml` | Built-in | 11 |
| State | `@startuml` | Graphviz | 13 |
| Component / Deployment | `@startuml` | Graphviz | 11 |
| Use Case | `@startuml` | Graphviz | 3 |
| Object | `@startuml` | Graphviz | 4 |
| Timing | `@startuml` | Built-in | 2 |
| ERD (Chen) | `@startchen` | Graphviz | 6 |
| Gantt | `@startgantt` | Built-in | 1 |
| JSON | `@startjson` | Built-in | 1 |
| YAML | `@startyaml` | Built-in | 1 |
| Mindmap | `@startmindmap` | Built-in | 1 |
| WBS | `@startwbs` | Built-in | 5 |
| NWDiag | `@startnwdiag` | Built-in | 1 |
| Salt / Wireframe | `@startsalt` | Built-in | 1 |
| DOT (Graphviz) | `@startdot` | Subprocess pass-through | 1 |
| EBNF | `@startebnf` | Built-in | 2 |
| Regex | `@startregex` | Built-in | 3 |
| BPM | `@startbpm` | Built-in | 4 |
| Board | `@startboard` | Built-in | 1 |
| Chronology | `@startchronology` | Built-in | 1 |
| Chart | `@startchart` | Built-in | 2 |
| Pie | `@startpie` | Built-in | 1 |
| HCL | `@starthcl` | Built-in | 1 |
| Flow | `@startflow` | Built-in | 2 |
| Wire | `@startwire` | Built-in | 2 |
| Archimate | `@startuml` | Graphviz | 2 |
| Packet | `@startpacket` | Built-in | 1 |

### Text / Passthrough Types

| Type | Start Tag | Ref Tests | Notes |
|------|-----------|-----------|-------|
| Creole | `@startcreole` | 1 | Rich text markup rendering |
| Def | `@startdef` | 1 | Plain text display |
| Math | `@startmath` | 1 | Formula placeholder (Java requires external tools) |
| LaTeX | `@startlatex` | 1 | Formula placeholder (Java requires external tools) |
| Git | `@startgit` | 2 | Git log visualization |
| Files | `@startfiles` | 2 | File tree display |

### Intentionally Unsupported

| Type | Start Tag | Reason |
|------|-----------|--------|
| DITAA | `@startditaa` | Java delegates to embedded third-party rasterizer (no SVG). Implementing ASCII-art → SVG from scratch is out of scope. |
| JCCKIT | `@startjcckit` | Java AWT-only charting library with no SVG mode. No Rust equivalent. |
| Project | `@startproject` | Java stable v1.2026.2 itself emits "not supported" error page for this type. |

## Preprocessor

Full preprocessor pipeline that expands all directives before parsing.

### Variables & Assignment
- `!$var = value` — variable assignment (three types: Str / Int / Array)
- `?=` conditional assignment
- `!local` local variables
- `!undef` undefine

### Conditionals
- `!if` / `!ifdef` / `!ifndef` / `!else` / `!elseif` / `!endif`
- Boolean logic: `&&`, `||`, `!`, parenthesized grouping

### Functions & Procedures
- `!function` / `!endfunction`
- `!procedure` / `!endprocedure`
- `!unquoted procedure`
- `!return` with expression evaluation
- Default parameter values
- `%call_user_func()` / `%invoke_procedure()` dynamic invocation

### Macros
- `!define NAME body`
- `!define NAME(params) body`
- `!definelong NAME` ... `!enddefinelong`

### Loops
- `!foreach $var in collection` ... `!endfor`
- `!while condition` ... `!endwhile` (10,000 iteration guard)
- Nested loops

### File Includes
- `!include path` — local relative path
- `!include <stdlib/module>` — built-in standard library
- `!include http://...` / `!includeurl` — remote URL
- `!include_once` / `!include_many`
- `!includesub file!PART` — sub-section extraction
- `!import archive.zip` — ZIP/JAR archive import

### Themes
- `!theme NAME` — built-in theme
- `!theme NAME from local/dir`
- `!theme NAME from <subdir>`
- `!theme NAME from https://...`

### Built-in Functions (35+)

`%strlen`, `%substr`, `%strpos`, `%splitstr`, `%splitstr_regex`, `%string`,
`%lower`, `%upper`, `%chr`, `%ord`, `%newline`, `%breakline`,
`%intval`, `%boolval`, `%not`, `%mod`, `%dec2hex`, `%hex2dec`,
`%size`, `%true`, `%false`,
`%variable_exists`, `%function_exists`,
`%get_variable_value`, `%set_variable_value`,
`%filename`, `%dirpath`, `%file_exists`, `%getenv`,
`%get_all_theme`, `%get_all_stdlib`

### Other
- `!pragma key value`
- `!assert condition`
- `!dump_memory` (compatibility stub)
- Line continuation (trailing `\`)
- Arithmetic expression evaluation (+, -, *, /, %, operator precedence, parentheses)

## Style System

### skinparam
- 30+ properties: BackgroundColor, FontColor, FontSize, FontName, BorderColor, ArrowColor, RoundCorner, etc.
- Element-level overrides: `skinparam classFontColor`, `skinparam sequenceArrowColor`, etc.
- Color normalization: `#RGB` → `#RRGGBB`, named colors, `transparent`
- Gradient support: `#color1|color2`, `#color1/color2`
- All diagram types are wired in

### Direction
- `left to right direction` / `top to bottom direction`
- Supported for Class, Sequence, Activity, State, Component, ERD, WBS

### Theme
- Built-in rose default theme (30 color-domain fields)
- SkinParams automatically fall back to theme defaults

## Rich Text / Creole Markup

### Inline Formatting
- `**bold**` / `<b>bold</b>`
- `//italic//` / `<i>italic</i>`
- `__underline__` / `<u>underline</u>`
- `~~strike~~` / `<s>strike</s>`
- `""monospace""`
- `<color:red>text</color>`
- `<size:18>text</size>`
- `<back:yellow>text</back>`
- `<font:courier>text</font>`
- `<sub>subscript</sub>` / `<sup>superscript</sup>`
- `~` escape character

### Block Elements
- `* item` — unordered list
- `# item` — ordered list
- `|= H | H |` / `| v | v |` — tables
- `----` — horizontal rule

### Links
- `[[url]]`
- `[[url label]]`
- `[[url{tooltip} label]]`

### Images & Icons
- `<img:path>` — embedded image reference
- `<&icon>` — OpenIconic icons (223 built-in icons)
- `<$sprite>` — custom SVG sprite reference

## SVG Sprite

- `sprite name <svg>...</svg>` — single-line / multi-line SVG definition
- `sprite $name <svg>...</svg>` — $ prefix is optional
- `<$name>` — reference sprite in text
- viewBox-aware scaling, inlined as `<g>` elements
- Supports complex SVG features: gradients, transforms, text styles, embedded images

## Sequence Diagram Features

### Participant Shapes
`participant`, `actor`, `boundary`, `control`, `entity`, `database`, `collections`, `queue`

### Combined Fragments
`alt/else`, `loop`, `opt`, `par`, `break`, `critical`, `group`, `ref over`

### Other
- Divider `==...==`
- Delay `...`
- `autonumber [start]`
- Participant colors
- Handwritten mode

## Activity Diagram Features

### Control Flow
- If / else / elseif branching
- While / repeat-while loops
- Fork / join parallel
- Goto / label jump
- Break exit
- Backward loops

### Swimlanes
- `|Swimlane|` syntax
- Multiple swimlanes rendered side by side
- Cross-swimlane L-shaped edge routing

## State Diagram Features

### Pseudo-states
- Fork / Join bars
- Choice diamond
- History `[H]` / Deep History `[H*]`

### Concurrent Regions
- `--` separator

## Metadata

- `title` / `title ... end title`
- `header` / `footer`
- `legend` / `legend ... end legend`
- `caption`

## Cross-diagram Features

- Note rendering: dog-ear polygon + dashed connectors (all diagram types)
- Hyperlinks / tooltips
- Handwritten mode (`skinparam handwritten true`) with Java-matching jiggle RNG
- Gradient fills (linear `|` / radial `/`)
- Source-seeded SVG IDs (deterministic output)
- Error handling: line/column tracking, error page generation
- CJK / Unicode character width calculation
- Multi-block PUML rendering
- Embedded subdiagram support

## Output Format

- **SVG** — the only output format

## Out of Scope

- PNG / PDF / EPS / ASCII and other output formats
- GUI / Web Server / FTP / Pipe modes
- PlantUML Server URL encoding/decoding
- Security sandbox
- ELK layout engine
- Full plantuml-stdlib (only vendored on demand)
- Full upstream theme catalog

## Test Coverage

| Category | Count |
|----------|-------|
| Unit tests | 2,693 |
| Integration tests | 185 |
| Reference tests (byte-exact) | 337 |
| Ignored (unsupported) | 3 |
| **Total** | **3,215** |
