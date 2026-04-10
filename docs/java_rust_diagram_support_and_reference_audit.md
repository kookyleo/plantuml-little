# Java/Rust Stable-Reference Audit

> Updated 2026-04-10.
> Authority: official PlantUML stable `v1.2026.2`.

## 1. Current Verdict

- Java authority checkout: `/ext/plantuml/plantuml-official-stable-v1.2026.2`
- Stable authority SHA: `bb8550d720e93f3e7f016a987848fb769e0222f5`
- Current results:
  - `cargo test --lib` -> `2681/2681`
  - `cargo test --test reference_tests` -> `329 passed / 0 failed / 3 ignored`
- The active harness resolves references through direct same-path lookup plus `tests/reference/INDEX.tsv` fallback.

Current ignored cases:

1. `tests/reference_tests.rs:986` — `ditaa/basic`: Java stable writes raw PNG bytes under `-tsvg`
2. `tests/reference_tests.rs:1024` — `jcckit/basic`: Java stable writes raw PNG bytes under `-tsvg`
3. `tests/reference_tests.rs:1708` — `sprite/svg2GroupsWithStyle`: Java stable throws `NullPointerException`

## 2. Meaning Of "Done"

For this repository, a diagram family is considered done only when all of the following hold:

1. Rust accepts the Java-stable start tag and syntax actually used by the fixture corpus.
2. Rust produces SVG, not an approximation or fallback-specific custom output.
3. The result is byte-exact against the checked-in stable Java authority SVG.
4. The case is exercised by `tests/reference_tests.rs` through either a direct same-path SVG or an `INDEX.tsv` mapping.

Under that definition, all Java stable families that currently provide UTF-8 SVG authority within the product boundary are done.

## 3. Stable Java Taxonomy vs Rust Status

| Stable Java type | Status | Notes |
|------------------|--------|-------|
| `UML` | Green | Active UML fixture corpus is byte-exact; one sprite fixture remains ignored because Java stable crashes |
| `BPM` | Green | Stable Java BPM mini-DSL is implemented and covered |
| `DITAA` | Blocked by Java output format | Java stable emits PNG bytes even when invoked with `-tsvg` |
| `DOT` | Green | Covered |
| `PROJECT` | Green | Stable Java behavior is an unsupported-release SVG page; Rust matches that page byte-exactly |
| `JCCKIT` | Blocked by Java output format | Java stable emits PNG bytes even when invoked with `-tsvg` |
| `SALT` | Green | Covered |
| `FLOW` | Green | `@startflow` is implemented and covered |
| `CREOLE` | Green | Covered |
| `MATH` | Green | Covered |
| `LATEX` | Green | Covered |
| `DEFINITION` | Green | Covered |
| `GANTT` | Green | Covered |
| `CHRONOLOGY` | Green | Covered |
| `NW` | Green | Covered |
| `MINDMAP` | Green | Covered |
| `WBS` | Green | Covered |
| `WIRE` | Green | Covered |
| `JSON` | Green | Covered |
| `GIT` | Green | Covered |
| `BOARD` | Green | Covered |
| `YAML` | Green | Covered |
| `HCL` | Green | Covered |
| `EBNF` | Green | Covered |
| `REGEX` | Green | Covered |
| `FILES` | Green | Covered |
| `CHEN_EER` | Green | Covered via Rust `Erd` |
| `CHART` | Green | Covered |
| `PACKET` | Green | Covered |
| `UNKNOWN` | Sentinel only | Not a product surface target |

## 4. Remaining Blockers

### 4.1 DITAA

- `DITAA` means "DIagrams Through Ascii Art".
- Java stable delegates this family to a raster path and writes PNG bytes to the output stream.
- Under `-tsvg`, the output file may still be named `.svg`, but the payload is PNG, not UTF-8 SVG.
- `plantuml-little` is intentionally SVG-only and returns `String`, so byte-exact parity is impossible without widening the product contract to binary outputs.

### 4.2 JCCKIT

- `JCCKIT` behaves the same way in Java stable for the current minimal fixture used here.
- The committed reference artifact is a `.svg` path whose content is PNG bytes emitted by Java stable.
- For the same reason as `DITAA`, this family cannot be made byte-exact inside the current SVG-only `String` API.

### 4.3 Java-Side Crash Fixture

- `tests/fixtures/sprite/svg2GroupsWithStyle.puml` is ignored because Java stable `v1.2026.2` throws `NullPointerException` on the authority side.
- This is not a Rust implementation gap.

## 5. Practical Conclusion

The strong statement

> "All implementable Java PlantUML diagram types are fully implemented in Rust and pass byte-exact alignment tests."

is now true under the repository's actual product boundary:

- SVG-only output
- public API returns `String`
- stable Java `v1.2026.2` is the authority

What remains unfinished is external to ordinary parity work:

1. `DITAA` — Java authority is PNG, not SVG
2. `JCCKIT` — Java authority is PNG, not SVG
3. `sprite/svg2GroupsWithStyle` — Java stable crashes
