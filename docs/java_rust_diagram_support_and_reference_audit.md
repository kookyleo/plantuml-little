# Java/Rust Stable-Reference Audit

> Updated 2026-04-10.
> Authority: official PlantUML stable `v1.2026.2`.

## 1. Current Verdict

- Java authority is `/ext/plantuml/plantuml-official-stable-v1.2026.2`.
- The current reference suite result is:
  - `cargo test --test reference_tests`
  - `326 passed / 0 failed / 2 ignored`
- The ignored cases are:
  - `tests/reference_tests.rs:1628` — Java stable NPE on `sprite/svg2GroupsWithStyle`
  - `tests/reference_tests.rs:973` — Java stable `ditaa` emits raw PNG bytes under `--svg`; impossible to byte-compare inside the SVG-only `String` API
- The active harness now consults `tests/reference/INDEX.tsv`, so indexed alternate-name references are genuinely byte-compared.
- Remaining coverage ambiguity is limited to fixtures for which stable Java still does not provide an SVG authority file, plus the `DITAA` binary-output blocker.

Therefore the statement

> "all implementable Java PlantUML diagram types are fully implemented in Rust and pass byte-exact alignment tests"

is **false**.

There are still three unfinished buckets:

1. stable Java diagram types that Rust does not implement yet
2. stable Java diagram types that Rust implements, but which are not fully covered by active byte-exact reference comparison in the current harness
3. stable Java paths whose authority output is incompatible with the product contract

## 2. Authoritative Evidence

Java stable taxonomy:

- `/ext/plantuml/plantuml-official-stable-v1.2026.2/src/main/java/net/sourceforge/plantuml/core/DiagramType.java`

Rust diagram entry points:

- `src/parser/common.rs`
- `src/parser/mod.rs`
- `src/model/diagram.rs`
- `src/render/svg.rs`

Current reference-test compare rule:

- `tests/reference_tests.rs:15`
- `tests/reference_tests.rs:17`
- `tests/reference_tests.rs:607`

Those lines matter because `load_reference()` now resolves references in two steps:

- direct same-path lookup under `tests/reference/`
- fallback lookup through `tests/reference/INDEX.tsv`

## 3. Stable Java Taxonomy vs Rust Status

Legend:

- `Implemented`: Rust has parse/layout/render support for this stable Java type.
- `Missing`: stable Java has the type, but Rust has no first-class support.
- `Coverage gap`: Rust has implementation, but current repo state does not put all fixture coverage for that type under active byte-exact comparison.

| Stable Java type | Status | Notes |
|------------------|--------|-------|
| `UML` | Implemented | Major UML subfamilies are implemented and the active suite is green, but one sequence fixture lacks direct reference coverage and one sprite fixture is ignored due to Java NPE |
| `BPM` | Implemented | Stable Java BPM mini-DSL is matched by Rust and covered by ref tests |
| `DITAA` | Authority-format blocker | Stable Java writes PNG bytes for this family even under `--svg`; Rust is intentionally SVG-only and returns `String` |
| `DOT` | Implemented | Direct parser/render path exists and is covered |
| `PROJECT` | Missing | No `@startproject` detection or parser path in current Rust entry points |
| `JCCKIT` | Missing | No `@startjcckit` detection or parser path in current Rust entry points |
| `SALT` | Implemented | Covered |
| `FLOW` | Missing | No `@startflow` detection or parser path in current Rust entry points |
| `CREOLE` | Implemented | Standalone `@startcreole` path exists and has a ref test |
| `MATH` | Implemented | Standalone `@startmath` path exists and has a ref test |
| `LATEX` | Implemented | Standalone `@startlatex` path exists and has a ref test |
| `DEFINITION` | Implemented | `@startdef` path exists and has a ref test |
| `GANTT` | Implemented | Covered |
| `CHRONOLOGY` | Implemented | Current fixture is byte-compared and green |
| `NW` | Implemented | Rust `Nwdiag` path exists and is covered |
| `MINDMAP` | Implemented | Covered |
| `WBS` | Implemented | Covered |
| `WIRE` | Implemented | `@startwire` path exists and both current wire fixtures have direct refs |
| `JSON` | Implemented | Covered |
| `GIT` | Implemented | Current fixtures are byte-compared and green |
| `BOARD` | Implemented | Current fixture is byte-compared and green |
| `YAML` | Implemented | Covered |
| `HCL` | Implemented | Covered |
| `EBNF` | Implemented | Covered |
| `REGEX` | Implemented | Covered |
| `FILES` | Implemented | Covered |
| `CHEN_EER` | Implemented | Implemented as Rust `Erd`; covered |
| `CHART` | Coverage gap | `bar_basic` and `single_series` are now byte-compared and green; `pie_basic` still has no stable-Java SVG |
| `PACKET` | Coverage gap | Implemented in Rust, but both current packet fixtures have no direct same-path reference SVG |
| `UNKNOWN` | Sentinel only | Not a product surface target |

## 4. Unfinished Diagram Types

This is the actionable list.

### 4.1 Not Implemented In Rust

These stable Java diagram types are still genuinely missing:

1. `PROJECT`
2. `JCCKIT`
3. `FLOW`

Root cause:

- `src/parser/common.rs` has no start-tag detection for these types.
- `src/parser/mod.rs` has no `DiagramHint` variants or parse dispatch for these types.
- `src/model/diagram.rs` has no corresponding `Diagram` variants.

### 4.2 Implemented, But Not Fully Under Byte-Exact Protection

These types are implemented in Rust, but the current repository state does not fully prove byte-exact parity for them:

1. `CHART`
2. `PACKET`

Why they are still unfinished from a parity-audit perspective:

- the current `reference_test!` macro only compares against a direct same-path SVG
- these fixtures currently do not have that direct same-path SVG in `tests/reference/`
- so those tests pass without executing a byte-for-byte SVG comparison

Concrete gaps:

- `tests/fixtures/chart/pie_basic.puml` -> missing `tests/reference/chart/pie_basic.svg`
- `tests/fixtures/packet/basic.puml` -> missing `tests/reference/packet/basic.svg`
- `tests/fixtures/packet/tcp.puml` -> missing `tests/reference/packet/tcp.svg`

### 4.3 Fixture-Level Byte-Exact Gaps Inside Otherwise Implemented Families

These are not missing top-level Java `DiagramType` values, but they still block the stronger claim that every covered family is fully protected by byte-exact tests:

- `tests/fixtures/pie/basic.puml` has no direct `tests/reference/pie/basic.svg`
- `tests/fixtures/sequence/seq_divider001.puml` is now byte-compared and green
- `tests/reference_tests.rs:1628` ignores `sprite/svg2GroupsWithStyle` because Java stable itself throws `NullPointerException`

### 4.4 Authority-Format Blocker

`DITAA` is a separate blocker from ordinary parser/render gaps.

- Official PlantUML stable `v1.2026.2` emits raw PNG bytes for `ditaa` even when invoked with `--svg` / `-tsvg`
- The current stable reference is `tests/reference/ditaa/-r.svg`, but the file content is PNG, not UTF-8 SVG
- `plantuml-little` is intentionally SVG-only, and its public API returns `String`
- Therefore true byte-exact parity for `DITAA` is impossible without changing the product contract

## 5. Practical Conclusion

If the question is:

> "Is Rust now green on the active reference suite?"

The answer is:

- yes, except for one intentionally ignored Java-crash fixture

If the question is:

> "Have all implementable Java PlantUML diagram types been completely implemented and proven by byte-exact comparison?"

The answer is:

- no

The remaining unfinished work is:

1. implement `PROJECT`
2. implement `JCCKIT`
3. implement `FLOW`
4. repair reference coverage so `DITAA`, `CHRONOLOGY`, `GIT`, `BOARD`, `CHART`, `PACKET`, `PIE`, and the `seq_divider001` sequence case are actually byte-compared by the harness
