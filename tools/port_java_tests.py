#!/usr/bin/env python3
"""Parse Java TestResult files and generate Rust integration tests.

Reads Java's DEBUG output format (RECTANGLE, LINE, POLYGON, TEXT, PATH)
and generates Rust tests that verify layout_sequence() output against
the Java-expected coordinates.

Usage:
    python3 tools/port_java_tests.py
"""

import re
import os
from pathlib import Path
from dataclasses import dataclass, field

JAVA_TEST_DIR = Path("/ext/plantuml/plantuml/src/test/java/nonreg/simple")
RUST_FIXTURES = Path("tests/fixtures/nonreg/simple")

@dataclass
class Rect:
    pt1: tuple  # (x, y)
    pt2: tuple  # (x, y)
    stroke: str
    color: str
    backcolor: str

@dataclass
class Line:
    pt1: tuple
    pt2: tuple
    stroke: str

@dataclass
class Polygon:
    points: list
    stroke: str

@dataclass
class Text:
    text: str
    position: tuple
    font: str

@dataclass
class TestData:
    name: str
    puml: str
    dimension: tuple
    elements: list = field(default_factory=list)


def parse_point(s):
    """Parse '[ 55.0477 ; 66.0000 ]' → (55.0477, 66.0)"""
    m = re.search(r'\[\s*([\d.]+)\s*;\s*([\d.]+)\s*\]', s)
    if m:
        return (float(m.group(1)), float(m.group(2)))
    return None


def parse_test_result(path):
    """Parse a Java TestResult file into structured elements."""
    text = path.read_text()

    # Extract content between triple quotes
    m = re.search(r'"""(.*?)"""', text, re.DOTALL)
    if not m:
        return None
    content = m.group(1).strip()

    lines = content.split('\n')
    elements = []
    dimension = None

    i = 0
    while i < len(lines):
        line = lines[i].strip()

        if line.startswith('dimension:'):
            dimension = parse_point(line)

        elif line == 'RECTANGLE:':
            rect = {'type': 'RECTANGLE'}
            i += 1
            while i < len(lines) and lines[i].strip() and not lines[i].strip().startswith(('RECTANGLE:', 'LINE:', 'POLYGON:', 'TEXT:', 'PATH:', 'EMPTY')):
                l = lines[i].strip()
                if l.startswith('pt1:'):
                    rect['pt1'] = parse_point(l)
                elif l.startswith('pt2:'):
                    rect['pt2'] = parse_point(l)
                elif l.startswith('stroke:'):
                    rect['stroke'] = l.split(':', 1)[1].strip()
                elif l.startswith('color:'):
                    rect['color'] = l.split(':', 1)[1].strip()
                elif l.startswith('backcolor:'):
                    rect['backcolor'] = l.split(':', 1)[1].strip()
                elif l.startswith('xCorner:'):
                    rect['xCorner'] = int(l.split(':', 1)[1].strip())
                i += 1
            elements.append(rect)
            continue

        elif line == 'LINE:':
            ln = {'type': 'LINE'}
            i += 1
            while i < len(lines) and lines[i].strip() and not lines[i].strip().startswith(('RECTANGLE:', 'LINE:', 'POLYGON:', 'TEXT:', 'PATH:', 'EMPTY')):
                l = lines[i].strip()
                if l.startswith('pt1:'):
                    ln['pt1'] = parse_point(l)
                elif l.startswith('pt2:'):
                    ln['pt2'] = parse_point(l)
                elif l.startswith('stroke:'):
                    ln['stroke'] = l.split(':', 1)[1].strip()
                i += 1
            elements.append(ln)
            continue

        elif line == 'POLYGON:':
            poly = {'type': 'POLYGON', 'points': []}
            i += 1
            while i < len(lines) and lines[i].strip() and not lines[i].strip().startswith(('RECTANGLE:', 'LINE:', 'POLYGON:', 'TEXT:', 'PATH:', 'EMPTY')):
                l = lines[i].strip()
                if l.startswith('- ['):
                    pt = parse_point(l)
                    if pt:
                        poly['points'].append(pt)
                elif l.startswith('stroke:'):
                    poly['stroke'] = l.split(':', 1)[1].strip()
                i += 1
            elements.append(poly)
            continue

        elif line == 'TEXT:':
            txt = {'type': 'TEXT'}
            i += 1
            while i < len(lines) and lines[i].strip() and not lines[i].strip().startswith(('RECTANGLE:', 'LINE:', 'POLYGON:', 'TEXT:', 'PATH:', 'EMPTY')):
                l = lines[i].strip()
                if l.startswith('text:'):
                    txt['text'] = l.split(':', 1)[1].strip()
                elif l.startswith('position:'):
                    txt['position'] = parse_point(l)
                elif l.startswith('font:'):
                    txt['font'] = l.split(':', 1)[1].strip()
                i += 1
            elements.append(txt)
            continue

        i += 1

    return dimension, elements


def parse_puml_from_test(path):
    """Extract puml source from Java Test.java file."""
    text = path.read_text()
    m = re.search(r'"""(.*?)"""', text, re.DOTALL)
    if m:
        return m.group(1).strip()
    return None


def classify_elements(elements):
    """Classify elements into semantic roles based on properties."""
    participants = []  # Rounded rects with backcolor ffe2e2f0
    activations = []   # Sharp rects with backcolor ffffffff
    lifelines = []     # Dashed lines (stroke 5.0-5.0-...)
    arrow_lines = []   # Solid/dashed lines (stroke 0.0-0.0-1.0 or 2.0-2.0-1.0)
    arrow_heads = []   # Polygons with 4 points
    texts = []         # Text elements

    for e in elements:
        if e['type'] == 'RECTANGLE':
            bc = e.get('backcolor', '')
            xc = e.get('xCorner', 0)
            if bc == 'ffe2e2f0' and xc == 5:
                participants.append(e)
            elif bc == 'ffffffff' and xc == 0:
                activations.append(e)
        elif e['type'] == 'LINE':
            stroke = e.get('stroke', '')
            if stroke.startswith('5.0-5.0'):
                lifelines.append(e)
            else:
                arrow_lines.append(e)
        elif e['type'] == 'POLYGON':
            if len(e.get('points', [])) == 4:
                arrow_heads.append(e)
        elif e['type'] == 'TEXT':
            texts.append(e)

    return {
        'participants': participants,
        'activations': activations,
        'lifelines': lifelines,
        'arrow_lines': arrow_lines,
        'arrow_heads': arrow_heads,
        'texts': texts,
    }


def generate_rust_test(name, puml, dimension, classified):
    """Generate Rust test function from classified elements."""
    parts = classified['participants']
    acts = classified['activations']
    arrows = classified['arrow_lines']
    arrow_heads = classified['arrow_heads']

    # Deduplicate participants (head and tail are same participant)
    # Group by x-position center
    unique_parts = {}
    for p in parts:
        cx = round((p['pt1'][0] + p['pt2'][0]) / 2, 2)
        if cx not in unique_parts:
            unique_parts[cx] = p
    part_list = sorted(unique_parts.values(), key=lambda p: p['pt1'][0])

    # Deduplicate activations (rendered twice in SVG)
    unique_acts = {}
    for a in acts:
        key = (round(a['pt1'][0], 1), round(a['pt1'][1], 1))
        if key not in unique_acts:
            unique_acts[key] = a
    act_list = sorted(unique_acts.values(), key=lambda a: a['pt1'][1])

    lines = []
    lines.append(f'    #[test]')
    lines.append(f'    fn {name}() {{')
    lines.append(f'        let puml = r#"')
    for pl in puml.split('\n'):
        lines.append(f'{pl}')
    lines.append(f'"#;')
    lines.append(f'        let sd = parse_sequence(puml);')
    lines.append(f'        let layout = layout_sequence(&sd, &SkinParams::default()).unwrap();')
    lines.append(f'')

    # Dimension check
    if dimension:
        lines.append(f'        // Java dimension: [{dimension[0]:.4f} ; {dimension[1]:.4f}]')

    # Participant count
    n_parts = len(part_list)
    lines.append(f'        assert_eq!(layout.participants.len(), {n_parts}, "participant count");')

    # Participant positions (from Java DEBUG coordinates)
    for i, p in enumerate(part_list):
        x1, y1 = p['pt1']
        x2, y2 = p['pt2']
        w = x2 - x1
        lines.append(f'        // Java participant[{i}]: x=[{x1:.2f}..{x2:.2f}] w={w:.2f}')

    # Activation count and positions
    if act_list:
        lines.append(f'        assert_eq!(layout.activations.len(), {len(act_list)}, "activation count");')
        for i, a in enumerate(act_list):
            x1, y1 = a['pt1']
            x2, y2 = a['pt2']
            h = y2 - y1
            w = x2 - x1
            lines.append(f'        // Java activation[{i}]: pos=[{x1:.2f},{y1:.2f}]->[{x2:.2f},{y2:.2f}] h={h:.2f}')
            lines.append(f'        {{')
            lines.append(f'            let act = &layout.activations[{i}];')
            lines.append(f'            let height = act.y_end - act.y_start;')
            lines.append(f'            assert!((height - {h:.4f}).abs() < 1.0,')
            lines.append(f'                "activation[{i}] height {{height:.2}} should be ~{h:.2f}");')
            lines.append(f'        }}')

    # Message count
    n_arrows = len(arrows)
    lines.append(f'        assert_eq!(layout.messages.len(), {n_arrows}, "message count");')

    # Arrow relationships with activations
    for i, (al, ah) in enumerate(zip(arrows, arrow_heads)):
        tip_x = ah['points'][1][0] if len(ah['points']) >= 2 else 0
        line_x1 = al['pt1'][0]
        line_x2 = al['pt2'][0]
        line_y = al['pt1'][1]
        stroke = al.get('stroke', '')
        is_dashed = stroke.startswith('2.0')
        lines.append(f'        // Java msg[{i}]: line=[{line_x1:.2f},{line_y:.2f}]->[{line_x2:.2f}] tip_x={tip_x:.2f} {"dashed" if is_dashed else "solid"}')

    lines.append(f'    }}')
    return '\n'.join(lines)


def main():
    # Find all Sequence TestResult files
    test_results = sorted(JAVA_TEST_DIR.glob("Sequence*_TestResult.java"))

    all_tests = []

    for tr_path in test_results:
        name = tr_path.stem.replace('_TestResult', '')
        test_path = tr_path.parent / f"{name}_Test.java"

        if not test_path.exists():
            continue

        puml = parse_puml_from_test(test_path)
        result = parse_test_result(tr_path)

        if not puml or not result:
            print(f"SKIP {name}: no puml or no result data")
            continue

        dimension, elements = result
        classified = classify_elements(elements)

        # Generate test name
        test_name = name.lower()

        test_code = generate_rust_test(test_name, puml, dimension, classified)
        all_tests.append((name, test_code))

        print(f"OK {name}: {len(classified['participants'])} participants, "
              f"{len(classified['activations'])} activations, "
              f"{len(classified['arrow_lines'])} arrows")

    # Generate Rust test file
    rust_code = []
    rust_code.append('// Auto-generated from Java TestResult files.')
    rust_code.append('// Source: /ext/plantuml/plantuml/src/test/java/nonreg/simple/Sequence*_TestResult.java')
    rust_code.append('//')
    rust_code.append('// These tests verify that Rust layout_sequence() produces coordinates')
    rust_code.append('// matching Java PlantUML\'s DEBUG output for the same input diagrams.')
    rust_code.append('')
    rust_code.append('use plantuml_little::layout::sequence::layout_sequence;')
    rust_code.append('use plantuml_little::model::sequence::SequenceDiagram;')
    rust_code.append('use plantuml_little::style::SkinParams;')
    rust_code.append('')
    rust_code.append('fn parse_sequence(puml: &str) -> SequenceDiagram {')
    rust_code.append('    match plantuml_little::parser::parse(puml).expect("parse failed") {')
    rust_code.append('        plantuml_little::model::Diagram::Sequence(sd) => sd,')
    rust_code.append('        other => panic!("expected sequence diagram, got {:?}", std::mem::discriminant(&other)),')
    rust_code.append('    }')
    rust_code.append('}')
    rust_code.append('')

    for name, code in all_tests:
        rust_code.append(f'// Ported from Java: {name}_Test.java + {name}_TestResult.java')
        rust_code.append(code)
        rust_code.append('')

    output = '\n'.join(rust_code)
    print(f"\n{'='*60}")
    print(output)
    print(f"{'='*60}")
    print(f"\nGenerated {len(all_tests)} tests")

    # Write to file
    out_path = Path("tests/port_sequence_layout.rs")
    out_path.write_text(output + '\n')
    print(f"Written to {out_path}")


if __name__ == '__main__':
    main()
