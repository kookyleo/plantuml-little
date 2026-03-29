# Current Failure Analysis And Action Guide

## Snapshot

- Current authority baseline from `python3 scripts/analyze_failures.py --quick`: `221 passed / 75 failed / 296 total`
- The older baseline in `docs/reference_alignment_execution_todo.md` is now stale and should be treated as historical context, not as the current authority view.
- The current worktree is mostly clean for this line of work; the visible unrelated local change is `src/model/sequence.rs`.

## Important Execution Notes

- `scripts/analyze_failures.py --quick` is useful for suite-level clustering, but it still reads cached data from `scripts/failures.json`.
- Because of that, cache-based taxonomy is now partially stale for near-passing cases.
- Example: `state_note001` is no longer a viewport-height failure in live output. The first live diff is now an internal coordinate tail: `cx="152.998"` vs `153`.
- Example: `state_history001` has improved from earlier larger drifts, but still fails live at `448px` vs Java `404px`.
- Do not route final repairs directly from cached categories without a live focused rerun.

## Current Failure Shape

By diagram family:

- `SEQUENCE`: 34
- `DESCRIPTION`: 14
- `STATE`: 12
- `CLASS`: 11
- `CHEN_EER`: 4
- `ACTIVITY`: 3
- `MINDMAP`: 2
- `TIMING`: 2
- `GANTT`: 1

By surface symptom:

- Height drift still dominates.
- `newline_func` and `creole_markup` remain the strongest cross-cutting keywords.
- `SEQUENCE` is still the largest family, but not the fastest path to suite-wide gains.

## Core Diagnosis

The remaining failures should be split into four independent work queues.

### 1. Shared text-body semantics

This is still the best leverage point.

Why:

- It cuts across `DESCRIPTION`, `CLASS`, parts of `STATE`, and a subset of `SEQUENCE`.
- The strongest surviving keywords are still `newline_func` and `creole_markup`.
- Several mirrored failures likely collapse together once line splitting, creole expansion, and body-height accumulation are aligned.

Primary representatives:

- `tests/fixtures/component/jaws5.puml`
- `tests/fixtures/dev/jaws/jaws5.puml`
- `tests/fixtures/dev/jaws/jaws3.puml`
- `tests/fixtures/preprocessor/jaws3.puml`
- `tests/fixtures/dev/newline/deployment_on_name.puml`
- `tests/fixtures/misc/deployment_on_name.puml`

Primary files:

- `src/parser/creole.rs`
- `src/parser/class.rs`
- `src/parser/component.rs`
- `src/render/svg_richtext.rs`
- `src/render/svg.rs`

### 2. STATE tail cases

These are almost-through cases and should not be mixed with deep layout work.

Representative guards:

- `tests/fixtures/state/state_note001.puml`
- `tests/fixtures/state/state_fork001.puml`

Current status:

- `state_note001` is now a tiny internal geometry mismatch, not a gross viewport problem.
- This queue should be handled as precision cleanup after the deeper `STATE` chain is separated.

### 3. STATE deep composite-history chain

This remains a real bottom-layer gap.

Representative driver:

- `tests/fixtures/state/state_history001.puml`

Current status:

- Live failure is still `448px` vs Java `404px`.
- This is not a renderer constant problem.
- The unresolved problem is the Java top-level composite/history graphviz chain, especially how composite state, history pseudo-state, and parent feedback edge participate in layout.

Rule:

- Do not patch this from downstream SVG output.
- Do not mix this with `state_note001`.

### 4. SEQUENCE large family

This is still important, but should stay behind shared text-body work.

Why:

- It is the largest family.
- It is also the most expensive family to debug deeply.
- The current suite offers a faster cross-family path through shared text-body semantics first.

## What Changed Since Earlier Notes

- `state_note001` has materially improved and is now a tiny coordinate tail.
- `state_history001` is still broken, but the current failure scale is smaller than older runs.
- The old `176 / 120` documentation baseline no longer describes the active workspace.
- The custom diagnosis skill path under `~/.codex/skills/plantuml-reference-diagnosis` currently exists but its script files are missing. Do not assume that tool is immediately runnable until it is restored.

## Precision Action Guide

This is the next action order. Do not skip steps.

### Step 1. Freeze the new authority baseline

Run:

```bash
python3 scripts/analyze_failures.py --quick
```

Then immediately live-rerun these cases:

```bash
cargo test --test reference_tests reference_fixtures_state_state_note001_puml -- --nocapture
cargo test --test reference_tests reference_fixtures_state_state_history001_puml -- --nocapture
cargo test --test reference_tests reference_fixtures_component_jaws5_puml -- --nocapture
cargo test --test reference_tests reference_fixtures_dev_jaws_jaws5_puml -- --nocapture
cargo test --test reference_tests reference_fixtures_dev_jaws_jaws3_puml -- --nocapture
cargo test --test reference_tests reference_fixtures_dev_newline_deployment_on_name_puml -- --nocapture
```

Purpose:

- Reconcile cache taxonomy with live failures.
- Build the actual representative set for the next repair loop.

### Step 2. Prioritize shared text-body semantics

Start here, not with sequence, and not with `state_history001`.

Target cluster:

- `jaws5`
- `jaws3`
- `deployment_on_name`

Trace only these intermediate values first:

- visual line split result
- richtext or creole token expansion result
- per-line text width
- per-line line height
- accumulated body block height
- final text or body origin `y`

Primary question to answer:

- Where does Rust first diverge from Java: line splitting, token expansion, line metrics, or block stacking?

Repair rule:

- Fix the earliest divergence point in parser or shared text layout.
- Do not patch final SVG coordinates if the block model is already wrong upstream.

### Step 3. Split STATE into two queues

Queue A: near-pass cleanup

- `state_note001`
- `state_fork001`

Queue B: deep composite-history port

- `state_history001`
- `state_scxml0002`
- `state_scxml0003`

Do not debug these together.

Queue A objective:

- close 1px to 3px coordinate tails

Queue B objective:

- port Java composite-history graphviz semantics

### Step 4. Handle `state_history001` as a deep port

Before writing code, answer these exact questions:

1. Why does Java place `Paused` above the composite while Rust still yields a taller top-to-bottom chain?
2. In Java, where does `Active[H]` affect top-level layout: DOT structure, cluster participation, or post-dot coordinate normalization?
3. In Rust, which current approximation in `src/layout/state.rs` prevents the top-level composite/history graph from matching Java?

If those three answers are not explicit, do not patch `src/render/svg_state.rs`.

### Step 5. Restore or replace the diagnosis tooling

Current issue:

- The skill directory exists, but the previous trace scripts are not there.

Action:

- Either restore the `plantuml-reference-diagnosis` skill package
- Or stop referencing it and move all focused diagnosis back to repo-local scripts

Rule:

- Do not leave diagnosis tooling in a half-broken state.

### Step 6. Only then update the historical TODO

File:

- `docs/reference_alignment_execution_todo.md`

Do not update it until:

- the new authority baseline is frozen
- the live representative set is confirmed
- the next active work queues are explicit

## Recommended Immediate Next Repair

The next concrete repair target should be the shared text-body chain, not `state_history001`.

Recommended first live cluster:

- `component_jaws5`
- `dev_jaws_jaws5`
- `dev_jaws_jaws3`
- `dev_newline_deployment_on_name`

Reason:

- This path likely removes multiple failures across `DESCRIPTION` and `CLASS`.
- It is lower cost and higher leverage than diving directly into the still-deep `STATE` composite-history port.

## Short Priority Order

1. Freeze authority baseline and live representative set
2. Repair shared text-body semantics
3. Split `STATE` into tail queue and deep queue
4. Deep-port `state_history001` only after its Java chain is explicit
5. Revisit `SEQUENCE` after shared text-body gains land
