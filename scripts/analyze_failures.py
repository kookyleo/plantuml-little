#!/usr/bin/env python3
"""
Reference test failure root-cause analysis.

Usage:
    python3 scripts/analyze_failures.py          # full analysis
    python3 scripts/analyze_failures.py --quick   # skip per-test cargo runs, use cached data
    python3 scripts/analyze_failures.py --help

Requires: cargo test --test reference_tests to have been run at least once.
Output: structured report to stdout, machine-readable JSON to scripts/failures.json.
"""

import subprocess
import re
import json
import sys
import os
from collections import Counter, defaultdict
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parent.parent
CACHE_FILE = PROJECT_ROOT / "scripts" / "failures.json"


# ── Step 1: Run reference tests, collect pass/fail ─────────────────────

def run_reference_tests():
    """Run cargo test --test reference_tests, return (passed, failed_list, total)."""
    print("Running reference tests...", file=sys.stderr)
    r = subprocess.run(
        ["cargo", "test", "--test", "reference_tests"],
        capture_output=True, text=True, timeout=600,
        cwd=PROJECT_ROOT,
    )
    output = r.stdout + r.stderr

    passed_tests = []
    failed_tests = []
    for line in output.splitlines():
        m = re.match(r"test (\S+) \.\.\. (ok|FAILED)", line)
        if m:
            name = m.group(1)
            if m.group(2) == "ok":
                passed_tests.append(name)
            else:
                failed_tests.append(name)

    total_m = re.search(r"(\d+) passed; (\d+) failed", output)
    total = int(total_m.group(1)) + int(total_m.group(2)) if total_m else len(passed_tests) + len(failed_tests)
    return len(passed_tests), failed_tests, total


# ── Step 2: For each failed test, get the first diff context ───────────

def get_test_diff(test_name):
    """Run a single failing test, return dict with diff info."""
    r = subprocess.run(
        ["cargo", "test", "--test", "reference_tests", "--", test_name],
        capture_output=True, text=True, timeout=60,
        cwd=PROJECT_ROOT,
    )
    output = r.stdout + r.stderr

    info = {"test": test_name, "diagram_type": None, "expected": "", "actual": "", "puml_file": None}

    # Extract puml file path
    m = re.search(r'(tests/fixtures/\S+\.puml)', output)
    if m:
        info["puml_file"] = m.group(1)

    # Extract diagram type from the full test output (both expected and actual lines)
    m = re.search(r'data-diagram-type="(\w+)"', output)
    if m:
        info["diagram_type"] = m.group(1)

    # If not found in diff context, try to get it from the reference SVG directly
    if not info["diagram_type"] and info["puml_file"]:
        ref_path = PROJECT_ROOT / info["puml_file"].replace("fixtures/", "reference/").replace(".puml", ".svg")
        if ref_path.exists():
            ref_content = ref_path.read_text(errors="replace")[:2000]
            m = re.search(r'data-diagram-type="(\w+)"', ref_content)
            if m:
                info["diagram_type"] = m.group(1)

    # If still not found, try running the converter and checking output
    if not info["diagram_type"] and info["puml_file"]:
        puml_full = PROJECT_ROOT / info["puml_file"]
        if puml_full.exists():
            try:
                r2 = subprocess.run(
                    ["cargo", "run", "--release", "--quiet", "--", str(puml_full)],
                    capture_output=True, text=True, timeout=30, cwd=PROJECT_ROOT,
                )
                m = re.search(r'data-diagram-type="(\w+)"', r2.stdout)
                if m:
                    info["diagram_type"] = m.group(1)
            except Exception:
                pass

    # Extract expected/actual context
    for line in output.splitlines():
        if line.startswith("expected:"):
            info["expected"] = line
        elif line.startswith("actual:"):
            info["actual"] = line

    # Extract full viewport dimensions from reference SVG (not just first-diff context)
    if info["puml_file"]:
        ref_path = PROJECT_ROOT / info["puml_file"].replace("fixtures/", "reference/").replace(".puml", ".svg")
        if ref_path.exists():
            ref_head = ref_path.read_text(errors="replace")[:3000]
            wh = re.search(r'width:(\d+)px;height:(\d+)px', ref_head)
            if wh:
                info["ref_w"] = int(wh.group(1))
                info["ref_h"] = int(wh.group(2))
            else:
                wh = re.search(r'width="(\d+)px".*?height="(\d+)px"', ref_head)
                if wh:
                    info["ref_w"] = int(wh.group(1))
                    info["ref_h"] = int(wh.group(2))

    # Extract our viewport dimensions from actual SVG output.
    # The test diff context may not contain the viewport — check the actual: line,
    # but also look for height="NNpx" in the full output (which includes both expected/actual).
    for pattern_src in [info["actual"], output]:
        wh = re.search(r'width:(\d+)px;height:(\d+)px', pattern_src)
        if wh:
            info["our_w"] = int(wh.group(1))
            info["our_h"] = int(wh.group(2))
            break

    # If viewport wasn't in the diff context, extract from the actual: line height attr
    if "our_h" not in info:
        # The test output contains expected/actual lines with SVG snippets
        act = info.get("actual", "")
        hm = re.search(r'height="(\d+)px"', act)
        if hm:
            info["our_h"] = int(hm.group(1))
        # Also try to extract from full output (look for actual line's height)
        for line in output.splitlines():
            if line.startswith("actual:"):
                hm = re.search(r'height="(\d+)px"', line)
                if hm:
                    info["our_h"] = int(hm.group(1))
                wm = re.search(r'width="(\d+)px"', line)
                if wm:
                    info["our_w"] = int(wm.group(1))

    # If we have ref dimensions and the first diff is at height, infer our height
    # from the actual line showing the diff
    if "our_h" not in info and "ref_h" in info:
        # Parse dh from the classified diff
        exp = info.get("expected", "")
        act = info.get("actual", "")
        exp_hm = re.search(r'height="(\d+)', exp)
        act_hm = re.search(r'height="(\d+)', act)
        if exp_hm and act_hm:
            info["our_h"] = int(act_hm.group(1))

    return info


# ── Step 3: Classify the diff into a root cause ───────────────────────

def classify_diff(info):
    """Return (category, subcategory, detail_dict)."""
    exp = info["expected"]
    act = info["actual"]

    detail = {}

    # --- viewport width/height in style attr ---
    exp_wh = re.search(r'width:(\d+)px;height:(\d+)px', exp)
    act_wh = re.search(r'width:(\d+)px;height:(\d+)px', act)
    if exp_wh and act_wh:
        ew, eh = int(exp_wh.group(1)), int(exp_wh.group(2))
        aw, ah = int(act_wh.group(1)), int(act_wh.group(2))
        detail["exp_w"], detail["exp_h"] = ew, eh
        detail["act_w"], detail["act_h"] = aw, ah
        detail["dw"], detail["dh"] = aw - ew, ah - eh
        if ew != aw and eh != ah:
            return "viewport_both", _size_bucket(max(abs(aw - ew), abs(ah - eh))), detail
        elif ew != aw:
            return "viewport_width", _size_bucket(abs(aw - ew)), detail
        elif eh != ah:
            return "viewport_height", _size_bucket(abs(ah - eh)), detail

    # --- SVG root height attr ---
    exp_h = re.search(r'height="(\d+)px"', exp)
    act_h = re.search(r'height="(\d+)px"', act)
    if exp_h and act_h:
        eh, ah = int(exp_h.group(1)), int(act_h.group(1))
        detail["exp_h"], detail["act_h"], detail["dh"] = eh, ah, ah - eh
        if eh != ah:
            return "svg_height", _size_bucket(abs(ah - eh)), detail

    # --- specific attr diffs ---
    if "stroke-width" in exp and "stroke-width" in act:
        return "attr_stroke_width", "", detail
    if "<title>" in exp or "..http" in exp:
        return "url_tooltip", "", detail
    if "linearGradient" in exp or "radialGradient" in exp:
        return "sprite_gradient", "", detail
    if "transform=" in exp:
        return "sprite_transform", "", detail
    if re.search(r'[xy][12]?="[\d.]+"', exp):
        return "coordinate", "", detail
    if "<text" in exp:
        return "text_rendering", "", detail
    if 'fill="' in exp or "style=" in exp:
        return "style_attr", "", detail
    if "<rect" in exp or "<line" in exp or "<polygon" in exp:
        return "shape_element", "", detail
    if "<g " in exp or 'id="' in exp:
        return "structure", "", detail

    return "other", "", detail


def _size_bucket(delta):
    if delta <= 2:
        return "tiny(<=2px)"
    if delta <= 5:
        return "small(3-5px)"
    if delta <= 30:
        return "medium(6-30px)"
    return "large(>30px)"


# ── Step 4: Check puml content for keywords ────────────────────────────

def read_puml_keywords(puml_path):
    """Read puml file, return set of keywords found."""
    keywords = set()
    if not puml_path:
        return keywords
    full = PROJECT_ROOT / puml_path
    if not full.exists():
        return keywords
    content = full.read_text(errors="replace").lower()
    checks = {
        "teoz": "!pragma teoz" in content,
        "theme": "!theme " in content,
        "maxmessagesize": "maxmessagesize" in content,
        "creole_markup": any(m in content for m in ["**", "//", "__", "~~", '""', "<size:", "<color:", "<back:"]),
        "sprite": "<$" in content or "sprite " in content,
        "url_link": "[[" in content and "]]" in content,
        "newline_func": "%newline()" in content or "\\n" in content,
        "skinparam_font": "fontname" in content or "fontsize" in content,
        "handwritten": "handwritten" in content,
        "left_self_msg": re.search(r'\w+\s*<-+\s*\w+.*:\s*\w+', content) is not None
            and re.search(r'(\w+)\s*<-+\s*\1', content) is not None,
    }
    return {k for k, v in checks.items() if v}


# ── Step 5: Assemble the report ───────────────────────────────────────

def build_report(passed, failed_list, total, diffs):
    """Build structured report dict."""
    report = {
        "summary": {"total": total, "passed": passed, "failed": len(failed_list)},
        "by_diagram_type": Counter(),
        "by_root_cause": Counter(),
        "by_cause_and_type": defaultdict(Counter),
        "details": [],
    }

    for info in diffs:
        dtype = info["diagram_type"] or "UNKNOWN"
        cat, subcat, detail = classify_diff(info)
        keywords = read_puml_keywords(info.get("puml_file"))
        label = f"{cat}/{subcat}" if subcat else cat

        report["by_diagram_type"][dtype] += 1
        report["by_root_cause"][label] += 1
        report["by_cause_and_type"][label][dtype] += 1
        report["details"].append({
            "test": info["test"].replace("reference_fixtures_", ""),
            "diagram_type": dtype,
            "cause": label,
            "keywords": sorted(keywords),
            "ref_w": info.get("ref_w"),
            "ref_h": info.get("ref_h"),
            "our_w": info.get("our_w"),
            "our_h": info.get("our_h"),
            **detail,
        })

    return report


def print_report(report):
    """Pretty-print the report."""
    s = report["summary"]
    print(f"{'='*70}")
    print(f"  Reference Test Failure Analysis")
    print(f"  {s['passed']} passed / {s['failed']} failed / {s['total']} total")
    print(f"{'='*70}")

    print(f"\n{'─'*70}")
    print(f"  BY DIAGRAM TYPE")
    print(f"{'─'*70}")
    for dtype, count in sorted(report["by_diagram_type"].items(), key=lambda x: -x[1]):
        print(f"  {count:4d}  {dtype}")

    print(f"\n{'─'*70}")
    print(f"  BY ROOT CAUSE")
    print(f"{'─'*70}")
    for cause, count in sorted(report["by_root_cause"].items(), key=lambda x: -x[1]):
        types = report["by_cause_and_type"][cause]
        type_str = ", ".join(f"{t}:{c}" for t, c in sorted(types.items(), key=lambda x: -x[1])[:5])
        print(f"  {count:4d}  {cause}")
        print(f"        [{type_str}]")

    # Special breakdowns
    print(f"\n{'─'*70}")
    print(f"  KEYWORD ANALYSIS (puml content tags)")
    print(f"{'─'*70}")
    kw_counts = Counter()
    kw_by_cause = defaultdict(Counter)
    for d in report["details"]:
        for kw in d["keywords"]:
            kw_counts[kw] += 1
            kw_by_cause[kw][d["cause"]] += 1
    for kw, count in kw_counts.most_common():
        causes = ", ".join(f"{c}:{n}" for c, n in kw_by_cause[kw].most_common(3))
        print(f"  {count:4d}  {kw}  [{causes}]")

    # Height deltas distribution for height failures
    print(f"\n{'─'*70}")
    print(f"  HEIGHT DELTA DISTRIBUTION (for height-related failures)")
    print(f"{'─'*70}")
    deltas_by_type = defaultdict(list)
    for d in report["details"]:
        if "dh" in d and d["dh"] != 0:
            deltas_by_type[d["diagram_type"]].append(d["dh"])
    for dtype in sorted(deltas_by_type, key=lambda x: -len(deltas_by_type[x])):
        ds = sorted(deltas_by_type[dtype])
        print(f"  {dtype:12s} ({len(ds):3d}): min={ds[0]:+5d}  median={ds[len(ds)//2]:+5d}  max={ds[-1]:+5d}")

    # Sequence-specific: teoz vs non-teoz
    print(f"\n{'─'*70}")
    print(f"  SEQUENCE DIAGRAM BREAKDOWN")
    print(f"{'─'*70}")
    seq_teoz = [d for d in report["details"] if d["diagram_type"] == "SEQUENCE" and "teoz" in d["keywords"]]
    seq_non_teoz = [d for d in report["details"] if d["diagram_type"] == "SEQUENCE" and "teoz" not in d["keywords"]]
    print(f"  Teoz:     {len(seq_teoz):3d} tests")
    print(f"  Non-teoz: {len(seq_non_teoz):3d} tests")
    if seq_non_teoz:
        # Sub-categorize non-teoz
        sub_kw = Counter()
        for d in seq_non_teoz:
            if d["keywords"]:
                for kw in d["keywords"]:
                    sub_kw[kw] += 1
            else:
                sub_kw["(no special keywords)"] += 1
        print(f"  Non-teoz keyword breakdown:")
        for kw, c in sub_kw.most_common():
            print(f"    {c:3d}  {kw}")

    # Width-only failures
    width_only = [d for d in report["details"] if d["cause"].startswith("viewport_width")]
    if width_only:
        print(f"\n{'─'*70}")
        print(f"  WIDTH-ONLY FAILURES ({len(width_only)} tests)")
        print(f"{'─'*70}")
        for d in sorted(width_only, key=lambda x: abs(x.get("dw", 0)), reverse=True):
            dw = d.get("dw", 0)
            print(f"  {dw:+5d}px  {d['diagram_type']:12s}  {d['test']}")
            if d["keywords"]:
                print(f"          keywords: {', '.join(d['keywords'])}")

    # Nearly-passing tests: show tests sorted by "total error" (|dw|+|dh|)
    # to identify the best targets for the next fix loop
    print(f"\n{'─'*70}")
    print(f"  NEARLY PASSING (sorted by total dimensional error)")
    print(f"{'─'*70}")
    scored = []
    for d in report["details"]:
        rw = d.get("ref_w")
        rh = d.get("ref_h")
        ow = d.get("our_w")
        oh = d.get("our_h")
        dw = (ow - rw) if (rw is not None and ow is not None) else None
        dh = (oh - rh) if (rh is not None and oh is not None) else None
        if dw is not None and dh is not None:
            total_err = abs(dw) + abs(dh)
            scored.append((total_err, dw, dh, "both", d))
        elif dh is not None:
            scored.append((abs(dh), 0, dh, "h_only", d))
        # skip tests where we can't extract either dimension
    scored.sort(key=lambda x: x[0])
    for total_err, dw, dh, kind, d in scored[:30]:
        if kind == "both":
            w_str = f"w{dw:+d}" if dw != 0 else "w=ok"
            h_str = f"h{dh:+d}" if dh != 0 else "h=ok"
        else:
            w_str = "w=?"
            h_str = f"h{dh:+d}" if dh != 0 else "h=ok"
        kws = ",".join(d["keywords"][:3]) if d["keywords"] else ""
        marker = ""
        if kind == "both":
            if dw == 0 and dh != 0: marker = " [height-only]"
            elif dh == 0 and dw != 0: marker = " [width-only]"
            elif dw == 0 and dh == 0: marker = " [dims-match!]"
        else:
            marker = " [width-unknown]"
        print(f"  {total_err:5d}  {w_str:>7s} {h_str:>7s}  {d['diagram_type']:12s}  {d['test']}{marker}")
        if kws:
            print(f"                                          {kws}")


# ── Main ──────────────────────────────────────────────────────────────

def main():
    quick = "--quick" in sys.argv
    if "--help" in sys.argv:
        print(__doc__)
        return

    passed, failed_list, total = run_reference_tests()
    print(f"Collected {passed} passed, {len(failed_list)} failed out of {total}", file=sys.stderr)

    if quick and CACHE_FILE.exists():
        print(f"Loading cached diffs from {CACHE_FILE}", file=sys.stderr)
        with open(CACHE_FILE) as f:
            cached = json.load(f)
        diffs = cached["diffs"]
    else:
        print(f"Collecting per-test diffs for {len(failed_list)} tests...", file=sys.stderr)
        diffs = []
        for i, test in enumerate(failed_list):
            if (i + 1) % 20 == 0:
                print(f"  {i+1}/{len(failed_list)}...", file=sys.stderr)
            diffs.append(get_test_diff(test))

        # Cache results
        with open(CACHE_FILE, "w") as f:
            json.dump({"passed": passed, "failed": len(failed_list), "total": total, "diffs": diffs},
                      f, indent=2)
        print(f"Cached to {CACHE_FILE}", file=sys.stderr)

    report = build_report(passed, failed_list, total, diffs)
    print_report(report)


if __name__ == "__main__":
    main()
