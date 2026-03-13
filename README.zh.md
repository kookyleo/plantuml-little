# plantuml-little

[PlantUML](https://plantuml.com/) 的轻量级 Rust 实现，专注于将 `.puml` 文件转换为 SVG 输出。

## 目标

- **输入**: `.puml` 文件 (PlantUML 文本)
- **输出**: 仅 `.svg` 文件
- **形态**: 库 (`lib`) + CLI 可执行文件
- **布局**: [Graphviz](https://graphviz.org/) via vizoxide (Class / State / Component / ERD / UseCase) + 自有引擎 (Sequence / Activity / Timing / Gantt / JSON / Mindmap / WBS / Salt / DITAA / NWDiag)

## 支持的图表类型 (17 种)

Class, Sequence, Activity v3, State, Component/Deployment, Use Case, Object,
Timing, ERD (Chen), Gantt, JSON, YAML, Mindmap, WBS, DITAA, NWDiag, Salt/Wireframe,
DOT (Graphviz 透传)

## 功能特性

- 完整预处理器：变量、函数、条件、循环、包含、主题、35+ 内置函数
- Skinparam 样式系统，内置 rose 默认主题
- Creole 富文本标记（粗体 / 斜体 / 链接 / 列表 / 表格 / 颜色 / 字体）
- SVG Sprite 内联嵌入
- Sequence 组合片段、State 伪状态、Activity 泳道
- CJK / Unicode 文字宽度支持
- 错误报告含行号 / 列号定位

详见 [FEATURES.zh.md](FEATURES.zh.md) 完整支持清单。

## 用法

```bash
# CLI
plantuml-little input.puml -o output.svg

# 库
let svg = plantuml_little::convert(puml_source)?;
```

## 前置条件

- Rust 1.70+
- Graphviz (`apt install graphviz` / `brew install graphviz`)

## 不在范围内

- GUI、Web Server、FTP、Pipe 模式
- SVG 以外的输出格式
- PlantUML Server URL 编解码
- 安全沙箱系统

## 测试覆盖

1,319 单元测试 + 183 集成测试 = **1,502 测试**，296 个 fixture 文件，0 warnings。

## 致谢

本项目是 [PlantUML](https://plantuml.com/) 的独立 Rust 重新实现，原作者为 Arnaud Roques。我们对 PlantUML 团队在 diagram-as-code 领域的贡献深表敬意。本项目完全跟进 PlantUML 的 License 方案。

我们会不定期跟进上游的更新，所有规范性内容以上游为标准。欢迎提 Issue 和 PR。

## 许可证

PlantUML 采用多许可证开源方案。作为遵循相同语言规范的重新实现，本项目同样采用多许可证方式 — 你可以选择最适合的一种：

- [GPL 许可证](https://www.gnu.org/licenses/gpl-3.0.html)
- [LGPL 许可证](https://www.gnu.org/licenses/lgpl-3.0.html)
- [Apache 许可证](https://www.apache.org/licenses/LICENSE-2.0)
- [Eclipse Public 许可证](https://www.eclipse.org/legal/epl-2.0/)
- [MIT 许可证](https://opensource.org/licenses/MIT)

上游许可详情参见 [PlantUML 许可 FAQ](https://plantuml.com/en/faq#ddbc9d04378ee462)。
