# Special Ext Reference Split Workflow

## Goal

When a single failing reference fixture contains multiple mixed symptoms, do not debug it as one blob.

Split it into several tiny external raw-reference cases, make each case isolate exactly one problem, then fix them one by one. After the split cases all pass, rerun the original reference fixture.

This workflow is for near-passing or mixed-symptom failures where:

- the original `.puml` already has mostly matching structure
- multiple differences are stacked together
- one large case makes Java vs Rust tracing noisy

It is especially useful for `STATE` tail cases.

## Core Idea

Turn this:

- one failing reference test
- several overlapping visible differences
- unclear first divergence

Into this:

- `N` split fixtures
- each fixture contains one symptom once
- each fixture is compared directly against Java raw SVG
- each fixture has a narrow diagnosis target

The split cases are not meant to replace the real reference suite. They are temporary diagnosis probes that let us localize the root cause with much less noise.

## When To Use It

Use this workflow when a failing case looks like:

- one small geometry tail plus metadata drift
- one ordering drift plus UID drift
- one line-number issue mixed with layout drift
- one original case that clearly contains several separable sub-problems

Do not use it when:

- the whole structure is wrong from the parser upward
- the case is already a single clean symptom
- a shared root cause across many fixtures is already obvious from suite-level analysis

## The Workflow

### 1. Enumerate the visible sub-cases

For the original failing fixture, write down the distinct observable symptoms.

Example from `state_monoline_03`:

- final node Y is off by `1px`
- top-level `<g>` output order is wrong
- raw `ent/lnk` IDs differ
- `data-source-line` differs after continuation handling

Important rule:

- separate symptoms, not guesses about root cause
- if two symptoms are really the same geometry chain, keep them together

### 2. Design one minimal fixture per symptom

For each symptom, create a `.puml` that contains only that problem once.

Rules for split fixture design:

- only one occurrence of the target symptom
- no extra states, notes, or transitions unless required
- keep the structure as small as possible
- prefer top-level cases before composite cases

Good split fixtures from this round:

- `state_ext_final_y_only.puml`
- `state_ext_order_no_final.puml`
- `state_ext_id_self_only.puml`
- `state_ext_source_line_only.puml`

Location:

- [`tests/ext_fixtures/state/`](/ext/plantuml/plantuml-little/tests/ext_fixtures/state)

### 3. Compare directly against Java raw output

Do not rely only on the normal reference harness here.

Use a special external raw-reference test that:

- renders the split fixture with Rust
- renders the same fixture with Java PlantUML
- optionally normalizes unrelated noise
- writes both raw and canonicalized artifacts for inspection

Current harness:

- [`tests/special_ext_reference_state_split.rs`](/ext/plantuml/plantuml-little/tests/special_ext_reference_state_split.rs)

Run it with:

```bash
cargo test --test special_ext_reference_state_split -- --ignored --nocapture
```

Artifacts are written under:

- [`tmp_debug/special-ext-ref/`](/ext/plantuml/plantuml-little/tmp_debug/special-ext-ref)

### 4. Normalize only the unrelated dimensions

Each split case should compare only the dimension it is meant to test.

Examples:

- for an order-only case, normalize entity/link IDs and strip `data-source-line`
- for an ID-only case, keep raw IDs but strip `data-source-line`
- for a source-line-only case, normalize IDs but keep `data-source-line`
- for a geometry-only case, normalize IDs and strip `data-source-line`

Rule:

- never normalize away the thing you are trying to fix

### 5. Debug one split case at a time

For each split case:

1. run the Java/Rust raw comparison
2. identify the first concrete divergence
3. trace the Rust chain to the producing module
4. trace the Java chain or infer from Java raw SVG when enough
5. fix the earliest divergence point
6. rerun only that split case

Do not jump back to the big original fixture after every tiny edit.

### 6. Fix at the source, not downstream

Typical fix locations:

- parser if the issue is line-number or explicit-vs-implicit state semantics
- layout if the issue is node size, graphviz participation, or coordinate chain
- renderer if the issue is emission order or UID assignment

Examples from this round:

- continuation line handling was fixed in parser/preproc
- top-level order and UID sequencing were fixed in renderer
- final state `1px` Y drift was fixed in layout by aligning final-node size with Java

### 7. Only after all split cases pass, rerun the original reference test

This is the close-the-loop step.

Run the original fixture only after the isolated cases are green:

```bash
cargo test --test reference_tests reference_fixtures_state_state_monoline_03_puml -- --nocapture
```

If the original still fails:

- either one sub-case was not isolated correctly
- or there is an additional coupled issue not captured by the split set

Then add one more split case instead of going back to blob-debugging.

## Worked Example: `state_monoline_03`

Original fixture:

- [`tests/fixtures/state/state_monoline_03.puml`](/ext/plantuml/plantuml-little/tests/fixtures/state/state_monoline_03.puml)

Split into four isolated problems:

1. `source-line-only`
2. `order-only`
3. `id-only`
4. `final-y-only`

Observed repair order:

1. fix continuation-line physical line preservation
2. fix Java-like top-level state emission order
3. fix Java-like shared UID sequence for entities and links
4. fix final-state layout size so end-node Y matches Java

After those passed, the original reference case passed as well.

## Why This Works

Because it converts one ambiguous mismatch into several deterministic probes:

- each failure message becomes short
- each case points to one subsystem
- unrelated dimensions can be normalized away
- Java/Rust diffs become readable
- the final original-case validation remains strict

It also reduces the chance of making a downstream patch that accidentally hides a deeper divergence.

## Practical Rules

- Keep split fixtures tiny.
- Each split case should fail for one reason.
- Normalize unrelated noise, but only for the split case.
- Preserve Java-first reasoning.
- Fix the earliest divergence.
- Do not delete the original reference fixture.
- Do not declare victory until the original reference test passes.

## Current Commands

Run all split probes:

```bash
cargo test --test special_ext_reference_state_split -- --ignored --nocapture
```

Run the original repaired case:

```bash
cargo test --test reference_tests reference_fixtures_state_state_monoline_03_puml -- --nocapture
```

## Recommended Reuse Pattern

For the next stubborn single-case mismatch:

1. enumerate sub-symptoms
2. create `N` minimal split fixtures
3. add or reuse a special ext raw-reference harness
4. normalize only unrelated dimensions per case
5. fix the split cases one by one
6. rerun the original reference test

If a future case is in another diagram family, keep the same method and just replace the harness and fixture directory.
