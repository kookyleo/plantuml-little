# Java vs Rust PlantUML Feature Gap Analysis

> Generated 2026-04-03. Baseline: 296/296 reference tests, 2617/2617 unit tests.

## 1. Diagram Type Coverage (17/30+)

### Fully Implemented (17)

| Type | Parser | Layout | Render | Notes |
|------|--------|--------|--------|-------|
| Class | ✓ | ✓ | ✓ | Including C4 via stdlib |
| Sequence (classic + teoz) | ✓ | ✓ | ✓ | Both puma and teoz modes |
| Activity (new + old) | ✓ | ✓ | ✓ | Old-style sync bars + graphviz |
| State | ✓ | ✓ | ✓ | Composites, history, concurrent regions |
| Component/Deployment | ✓ | ✓ | ✓ | All USymbol types |
| Use Case | ✓ | ✓ | ✓ | Routed through component pipeline |
| ERD (Chen) | ✓ | ✓ | ✓ | Via svek/graphviz pipeline |
| Timing | ✓ | ✓ | ✓ | Robust + concise tracks, skinparam fonts |
| Gantt | ✓ | ✓ | ✓ | Scale, dependencies, weekly printscale |
| Mindmap | ✓ | ✓ | ✓ | Java Tetris packing algorithm |
| WBS | ✓ | ✓ | ✓ | Work breakdown structure |
| JSON | ✓ | ✓ | ✓ | Tree visualization |
| YAML | ✓ | ✓ | ✓ | Tree visualization |
| Salt (UI) | ✓ | ✓ | ✓ | Wireframe mockups |
| Ditaa | ✓ | ✓ | ✓ | ASCII art to SVG |
| Nwdiag | ✓ | ✓ | ✓ | Network diagrams |
| DOT/GraphViz | ✓ | ✓ | ✓ | Passthrough |

### Not Implemented (13+)

| Type | Java Location | Complexity | Priority |
|------|--------------|------------|----------|
| **Object** | `objectdiagram/` | Low — shares Class infrastructure | High |
| **Regex** | `regexdiagram/` | Medium — custom visualization | Low |
| **Board** | `board/` | Medium | Low |
| **EBNF** | `ebnf/` | Medium — syntax diagram | Low |
| **Archimate** | via `descdiagram/` + stdlib | Low — reuses component pipeline | Medium |
| **BPM** | `bpm/` | Medium | Low |
| **Flow** | `flowdiagram/` | Medium | Low |
| **Wire** | `wire/` | Medium | Low |
| **Packet** | `wire/` (shared) | Medium | Low |
| **HCL** | (Hashicorp Config) | Medium | Low |
| **Git** | `gitlog/` | Medium | Low |
| **Files** | `filesdiagram/` | Low | Low |
| **Chart** | (bar/pie) | Medium | Low |
| **Math/LaTeX** | `math/` | High — external LaTeX dep | Low |

## 2. Preprocessor Coverage (95%+)

### Fully Supported
- `!include` / `!include_once` / `!include_many` / `!includesub` / `!includeurl` / `!includedef`
- `!define` / `!definelong` / `!undef` (simple + parameterized)
- `!ifdef` / `!ifndef` / `!if` / `!elseif` / `!else` / `!endif`
- `!function` / `!procedure` / `!return` / `!unquoted`
- `!while` / `!endwhile` / `!foreach` / `!endfor`
- `!theme` / `!pragma` / `!log` / `!assert` / `!dump_memory`
- Variable scoping: `$var`, `!global`, `!local`
- 40+ `%functions`: strlen, substr, strpos, intval, date, filename, dirpath, newline, chr, dec2hex, hex2dec, darken, lighten, is_dark, is_light, reverse_color, hsl_color, not, string, size, load_json, upper, lower, ord, mod, random, boolval, feature, getenv, function_exists, variable_exists, set_variable_value, get_variable_value, splitstr, splitstr_regex, etc.

### Missing
| Feature | Java | Notes |
|---------|------|-------|
| `!option` directive | ✓ | Configuration key-value |
| `%get_json_key` / `%get_json_keys` / `%get_json_type` | ✓ | JSON introspection |
| `%json_add` / `%json_merge` / `%json_remove` / `%json_set` | ✓ | JSON manipulation |
| `%str2json` | ✓ | String to JSON parse |
| `%logical*` (bitwise ops) | ✓ | AND, OR, XOR, NOT |
| `%filedate` | ✓ | File modification date |
| `%get_current_theme` / `%get_stdlib` | ✓ | Theme/stdlib introspection |
| `%left_align` / `%right_align` / `%tabulation` | ✓ | Text alignment |

## 3. Skinparam & Style Coverage (90%+)

### Fully Supported
- 100+ legacy skinparams via case-insensitive HashMap
- `<style>` CSS-like blocks with 29 PName properties
- `!theme` loading from built-in + custom paths
- `hide` / `show` commands (fields, methods, stereotypes)
- `left to right direction` / `top to bottom direction`
- Color: hex (#RGB, #RRGGBB, #RRGGBBAA), 100+ named colors, gradients
- Fonts: family, size, bold, italic, weight
- Arrow styles: dotted, dashed, bold, hidden, thickness
- Per-diagram: sequence.*, class.*, state.*, component.*, activity.*, timing.*, gantt.*

### Missing
| Feature | Notes |
|---------|-------|
| Explicit `skin rose` command | Functionally equivalent via Theme::rose() defaults |
| External directory theme loading | Hardcoded theme resolution |
| Dynamic CSS selector expansion | Limited macro in selectors |

## 4. Output & API Coverage

### Supported
- **Output**: SVG only
- **API**: `convert()`, `convert_with_base_dir()`, `convert_with_input_path()`
- **Metadata**: `data-diagram-type`, `data-source-line`, `data-qualified-name`, entity IDs, `<?plantuml-src?>`
- **Interactive**: `[[url]]`, `[[url{tooltip}]]`, `[[url{tooltip} label]]` → SVG `<title>` + `<a>`
- **Sprites**: SVG sprites, pixel sprites (16-level grayscale), stdlib, monochrome mode

### Missing
| Feature | Priority | Notes |
|---------|----------|-------|
| PNG output | High | Needs cairo/resvg dependency |
| PDF/EPS output | Medium | Needs additional backends |
| CLI `-pipe` mode | High | Stdin/stdout pipeline |
| CLI `-t` format flag | High | Format selection |
| CLI `-I` include path | Medium | Include directory search |
| CLI `-D` defines | Medium | Command-line variable definition |
| WASM target | Medium | Web embedding |
| C FFI | Low | Non-Rust integration |
| Map files (.cmapx) | Low | Image map for HTML |
| ASCII art output | Low | Text-mode rendering |
| `PLANTUML_INCLUDE_PATH` env var | Low | Environment-based include paths |

## 5. Implementation Priority Matrix

### Phase 1: High Value, Low Cost
1. Object diagram (reuses Class infrastructure)
2. CLI enhancements (-pipe, -t, -I, -D)
3. `!option` directive

### Phase 2: High Value, Medium Cost
4. PNG output (resvg crate)
5. Archimate (reuses Component pipeline)
6. JSON manipulation functions (%json_*)

### Phase 3: Medium Value
7. WASM target
8. Files diagram
9. Chart (pie/bar)
10. External theme directory loading

### Phase 4: Low Priority
11. Remaining diagram types (Regex, Board, EBNF, BPM, Flow, Wire, Packet, HCL, Git)
12. Math/LaTeX (external dependency)
13. ASCII art output
14. Map files
