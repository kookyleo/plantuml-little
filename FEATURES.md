# plantuml-little Feature Support

This document records the current feature coverage of the project, for user reference and maintenance tracking.

## Diagram Types -- 17 types

| Type | Start Tag | Layout Engine | Fixture Count |
|------|-----------|---------------|---------------|
| Class | `@startuml` | Graphviz | 14 |
| Sequence | `@startuml` | Built-in engine | 31 |
| Activity v3 | `@startuml` | Built-in engine | 8 |
| State | `@startuml` | Graphviz | 13 |
| Component / Deployment | `@startuml` | Graphviz | 10 |
| Use Case | `@startuml` | Graphviz | 3 |
| Object | `@startuml` | Graphviz | (reuses Class) |
| Timing | `@startuml` | Built-in engine | 2 |
| ERD (Chen) | `@startchen` | Graphviz | 5 |
| Gantt | `@startgantt` | Built-in engine | 1 |
| JSON | `@startjson` | Built-in engine | 1 |
| YAML | `@startyaml` | Built-in engine | 1 |
| Mindmap | `@startmindmap` | Built-in engine | 1 |
| WBS | `@startwbs` | Built-in engine | 5 |
| DITAA | `@startditaa` | Built-in engine | 1 |
| NWDiag | `@startnwdiag` | Built-in engine | 1 |
| Salt / Wireframe | `@startsalt` | Built-in engine | 1 |
| DOT (Graphviz) | `@startdot` | Subprocess pass-through | 1 |

## Preprocessor

Full preprocessor pipeline that expands all directives before parsing.

### Variables & Assignment
- `!$var = value` -- variable assignment (three types: Str / Int / Array)
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
- `!include path` -- local relative path
- `!include <stdlib/module>` -- built-in standard library
- `!include http://...` / `!includeurl` -- remote URL
- `!include_once` / `!include_many`
- `!includesub file!PART` -- sub-section extraction
- `!import archive.zip` -- ZIP/JAR archive import

### Themes
- `!theme NAME` -- built-in theme
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
- Color normalization: `#RGB` -> `#RRGGBB`, named colors, `transparent`
- All 17 diagram types are wired in

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
- `* item` -- unordered list
- `# item` -- ordered list
- `|= H | H |` / `| v | v |` -- tables
- `----` -- horizontal rule

### Links
- `[[url]]`
- `[[url label]]`
- `[[url{tooltip} label]]`

## SVG Sprite

- `sprite name <svg>...</svg>` -- single-line / multi-line SVG definition
- `sprite $name <svg>...</svg>` -- $ prefix is optional
- `<$name>` -- reference sprite in text
- viewBox-aware scaling, inlined as `<g>` elements
- Supports complex SVG features: gradients, transforms, text styles, embedded images

## Sequence Diagram Extensions

### Participant Shapes
`participant`, `actor`, `boundary`, `control`, `entity`, `database`, `collections`, `queue`

### Combined Fragments
`alt/else`, `loop`, `opt`, `par`, `break`, `critical`, `group`, `ref over`

### Other
- `divider ==...==`
- `delay ...`
- `autonumber [start]`
- Participant colors

## State Diagram Extensions

### Pseudo-states
- Fork / Join bars
- Choice diamond
- History `[H]` / Deep History `[H*]`

### Concurrent Regions
- `--` separator

## Activity Diagram Extensions

### Swimlanes
- `|Swimlane|` syntax
- Multiple swimlanes rendered side by side
- Cross-swimlane L-shaped edge routing

## Metadata

- `title` / `title ... end title`
- `header` / `footer`
- `legend` / `legend ... end legend`
- `caption`

## Cross-diagram Features

- Note rendering: dog-ear polygon + dashed connectors (all diagram types)
- Hyperlinks / tooltips
- Error handling: line number + column number positioning
- CJK / Unicode character width calculation
- SVG output validation

## Output Format

- **SVG** -- the only output format

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
| Unit Tests | 1,319 |
| Integration Tests | 183 |
| Test Fixtures | 296 |
| **Total** | **1,502** |
