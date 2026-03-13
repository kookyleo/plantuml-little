#!/usr/bin/env python3
"""
从 PlantUML Java 测试文件中提取 @startuml...@enduml 块，保存为 .puml fixture 文件。
"""

import os
import re
import sys
from pathlib import Path
from collections import defaultdict

JAVA_TEST_ROOT = Path("/ext/plantuml/plantuml/src/test/java")
FIXTURES_ROOT = Path("/ext/plantuml/plantuml-little/tests/fixtures")

# 支持的 @start 指令及对应分类
START_DIRECTIVES = {
    "startuml": None,       # 根据内容再判断
    "startchen": "erd",
    "startwbs": "wbs",
    "startmindmap": "mindmap",
    "startgantt": "gantt",
    "startjson": "json",
    "startyaml": "yaml",
    "startdot": "dot",
    "startditaa": "ditaa",
    "startdirt": "ditaa",
    "startsalt": "salt",
    "startboard": "board",
    "startmath": "math",
    "startlatex": "latex",
    "startchronology": "chronology",
    "starttimeline": "timeline",
    "startregex": "misc",
    "startcreole": "misc",
}

# @startuml 内容分类规则：通过内容特征判断图表类型
# 注意：有歧义的类型放后面（如 sequence 的通用箭头很泛）
CONTENT_CLASSIFIERS = [
    # 状态图特征（优先于 sequence，因为 state s1 --> s2 会被 sequence 箭头误判）
    ("state", [
        r"^\s*state\s+",
        r"^\s*\[\*\]\s*-+>",
        r"^\s*-+>\s*\[\*\]",
    ]),
    # 类图特征（优先于 sequence）
    ("class", [
        r"^\s*class\s+\w",
        r"^\s*interface\s+\w",
        r"^\s*abstract\s+class\s+",
        r"^\s*enum\s+\w",
        r"^\s*hide\s+<<",
    ]),
    # 对象图特征
    ("object", [
        r"^\s*object\s+\w",
    ]),
    # 活动图特征（优先于 sequence，: action ; 不是序列图）
    ("activity", [
        r"^\s*start\s*$",
        r"^\s*stop\s*$",
        r"^\s*:\s*.+;\s*$",
        r"^\s*if\s*\(.+\)\s*then",
        r"^\s*fork\s*$",
        r"^\s*split\s*$",
        r"^\s*repeat\s*$",
        r"^\s*while\s*\(",
    ]),
    # 组件/部署图特征
    ("component", [
        r"^\s*component\s+",
        r"^\s*package\s+",
        r"^\s*node\s+",
        r"^\s*database\s+",
        r"^\s*cloud\s+",
        r"^\s*artifact\s+",
        r"^\s*frame\s+",
        r"^\s*rectangle\s+",
        r"^\s*sprite\s+\$",
    ]),
    # 时序/时间线特征
    ("timing", [
        r"^\s*robust\s+",
        r"^\s*concise\s+",
        r"^\s*clock\s+",
        r"^\s*binary\s+",
    ]),
    # 用例图特征
    ("usecase", [
        r"^\s*usecase\s+",
        r"^\s*\([\w\s]+\)\s*<",
        r"^\s*:[\w\s]+:\s*--",
    ]),
    # 部署图/盐图特征
    ("deployment", [
        r"^\s*salt\b",
        r"^\s*\{[#!\+]",
    ]),
    # 序列图特征（放在最后，因为 -> 很通用）
    ("sequence", [
        r"^\s*participant\s+",
        r"^\s*actor\s+",
        r"^\s*activate\s+",
        r"^\s*deactivate\s+",
        r"^\s*autonumber",
        r"^\s*loop\s+",
        r"^\s*alt\s+",
        r"^\s*group\s+",
        r"^\s*boundary\s+",
        r"^\s*control\s+",
        r"^\s*collections\s+",
        r"^\s*queue\s+",
        r"^\s*\w[\w\s]*\s*-+>+\s*\w",   # actor -> actor（最通用，放最后）
    ]),
]


def classify_startuml(content: str) -> str:
    """根据 @startuml 内容判断图表类型"""
    lines = content.splitlines()
    for diagram_type, patterns in CONTENT_CLASSIFIERS:
        for pattern in patterns:
            for line in lines:
                if re.search(pattern, line, re.IGNORECASE):
                    return diagram_type
    return "misc"


def has_preprocessor(content: str) -> bool:
    """检查是否包含预处理器指令（以 ! 开头的行，排除注释）"""
    for line in content.splitlines():
        stripped = line.strip()
        if stripped.startswith('!') and not stripped.startswith("!'"):
            return True
    return False


def extract_diagrams_from_java(java_file: Path):
    """从 Java 文件中提取所有 @start...@end 块。返回 list of (start_directive, content)"""
    try:
        text = java_file.read_text(encoding="utf-8", errors="replace")
    except Exception as e:
        return []

    results = []

    # 模式1：在三引号块内的内容（nonreg/simple 风格）
    triple_quote_pattern = re.compile(r'"""\s*\n(.*?)\n\s*"""', re.DOTALL)
    in_triple = set()
    for m in triple_quote_pattern.finditer(text):
        block = m.group(1)
        # 在三引号块内查找所有 @start...@end
        for start_m, content in extract_from_block(block):
            results.append((start_m, content, "triple_quote"))
            in_triple.add((start_m, content))

    # 模式2：在注释块内的内容（dev/newline 风格，在 /* ... */ 之间，没有三引号包裹）
    comment_pattern = re.compile(r'/\*.*?\*/', re.DOTALL)
    for m in comment_pattern.finditer(text):
        comment_text = m.group(0)
        for start_m, content in extract_from_block(comment_text):
            # 避免重复（triple quote 内容也在注释块内）
            if (start_m, content) not in in_triple:
                results.append((start_m, content, "comment"))

    return results


def extract_from_block(block: str):
    """从文本块中提取 @start...@end 对"""
    # 匹配各种 @start/@end 对
    pattern = re.compile(
        r'(@start\w+(?:\s+\S+)?)\s*\n(.*?)\n\s*(@end\w+)',
        re.DOTALL | re.IGNORECASE
    )
    results = []
    for m in pattern.finditer(block):
        start_line = m.group(1).strip()
        body = m.group(2)
        end_line = m.group(3).strip()
        full_content = start_line + "\n" + body + "\n" + end_line
        # 提取指令名
        directive_match = re.match(r'@(start\w+)', start_line, re.IGNORECASE)
        if directive_match:
            directive = directive_match.group(1).lower()
            results.append((directive, full_content))
    return results


def get_category(directive: str, content: str) -> str:
    """根据指令和内容确定分类目录"""
    if directive in START_DIRECTIVES and START_DIRECTIVES[directive] is not None:
        return START_DIRECTIVES[directive]
    if directive == "startuml":
        return classify_startuml(content)
    return "misc"


def make_filename(java_file: Path, index: int, total: int) -> str:
    """根据 Java 文件名生成 fixture 文件名"""
    stem = java_file.stem
    # 去除 _Test, Test 等后缀
    stem = re.sub(r'_?[Tt]est(?:Result)?$', '', stem)
    stem = stem.strip('_').lower()
    # 将驼峰和大写转为 snake_case
    stem = re.sub(r'([A-Z])', r'_\1', stem).lower().strip('_')
    stem = re.sub(r'[^a-z0-9_]', '_', stem)
    stem = re.sub(r'_+', '_', stem).strip('_')
    if not stem:
        stem = "unknown"
    if total > 1:
        return f"{stem}_{index:02d}.puml"
    return f"{stem}.puml"


def ensure_unique_path(path: Path) -> Path:
    """确保文件路径唯一，若已存在则加数字后缀"""
    if not path.exists():
        return path
    stem = path.stem
    suffix = path.suffix
    parent = path.parent
    counter = 2
    while True:
        new_path = parent / f"{stem}_{counter}{suffix}"
        if not new_path.exists():
            return new_path
        counter += 1


def main():
    print("开始扫描 Java 测试文件...")

    # 收集所有 Java 文件
    java_files = list(JAVA_TEST_ROOT.rglob("*.java"))
    print(f"找到 {len(java_files)} 个 Java 文件")

    # 统计
    stats = defaultdict(int)
    preprocessor_count = 0
    total_extracted = 0
    skipped_empty = 0

    # 确保输出目录存在
    FIXTURES_ROOT.mkdir(parents=True, exist_ok=True)

    all_diagrams = []

    for java_file in java_files:
        diagrams = extract_diagrams_from_java(java_file)
        if not diagrams:
            continue
        # 过滤掉同一文件的重复（triple_quote 和 comment 可能重复）
        seen = set()
        unique_diagrams = []
        for directive, content, source in diagrams:
            key = content.strip()
            if key not in seen:
                seen.add(key)
                unique_diagrams.append((directive, content, source))
        if unique_diagrams:
            all_diagrams.append((java_file, unique_diagrams))

    print(f"有 @start...@end 内容的文件：{len(all_diagrams)} 个")

    for java_file, diagrams in all_diagrams:
        total_in_file = len(diagrams)
        for idx, (directive, content, source) in enumerate(diagrams, 1):
            # 过滤掉内容太短的（少于 3 行，只有 @start/@end 的空内容）
            lines = [l for l in content.splitlines() if l.strip()]
            if len(lines) < 3:
                skipped_empty += 1
                continue

            # 判断是否含预处理器指令
            is_preprocessor = has_preprocessor(content)

            # 确定分类
            if is_preprocessor:
                category = "preprocessor"
                preprocessor_count += 1
            else:
                category = get_category(directive, content)

            # 生成文件名
            filename = make_filename(java_file, idx, total_in_file)

            # 确定输出路径
            out_dir = FIXTURES_ROOT / category
            out_dir.mkdir(parents=True, exist_ok=True)
            out_path = ensure_unique_path(out_dir / filename)

            out_path.write_text(content + "\n", encoding="utf-8")
            stats[category] += 1
            total_extracted += 1

    print("\n提取完成！")
    print(f"总计提取：{total_extracted} 个 fixture 文件")
    print(f"跳过空内容：{skipped_empty} 个")
    print("\n按类型统计：")
    for cat, count in sorted(stats.items(), key=lambda x: -x[1]):
        print(f"  {cat:20s} {count:4d} 个")
    print(f"\n预处理器类（preprocessor）：{preprocessor_count} 个")

    return stats, total_extracted


if __name__ == "__main__":
    main()
