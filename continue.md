# plantuml-little 接续指南

## 当前状态 (2026-03-17)

- **Git**: main 分支，HEAD = `ad2d0df`，干净工作区
- **Unit tests**: 2405 passed, 0 failed
- **Reference tests**: 61 / 296 passed (20.6%)
- **Stash / 废弃分支**: 已全部清理

### 通过测试分布

| 类型 | 通过 | 失败 | 备注 |
|------|------|------|------|
| sequence | 17 | 14 | 宽度计算偏差，参与者间距 |
| class | 10 | 5 | hideshow/meta wrapping 问题 |
| sprite | 11 | 29 | 部分 font/gradient/transform 未处理 |
| nonreg | 8 | 61 | 镜像测试，跟随主类型 |
| dev | 4 | 27 | 同上 |
| preprocessor | 1 | 37 | 同上 |
| component | 0 | 11 | 未用 svek 布局 |
| activity | 0 | 13 | 未用 svek 布局 |
| state | 0 | 14 | 未用 svek 布局 |
| object | 1 | 0 | |
| ditaa | 1 | 0 | |
| erd | 0 | 5 | |
| usecase | 0 | 3 | |
| wbs | 0 | 5 | |
| 其他 | 8 | 8 | gantt/json/mindmap/nwdiag/salt/timing |

---

## 项目架构

### 底座模块（已实现，但大部分未接入 renderer）

```
src/klimt/        # 2D 图形抽象层（Java klimt 包移植）
  svg.rs          # SvgGraphic - SVG 元素生成（属性按 Java 字母序）
  color.rs        # HColor 颜色系统
  geom.rs         # 几何类型 (XPoint2D, XDimension2D, XRectangle2D)
  shape.rs        # 图形原语 (UPath, UStroke)
  font.rs         # StringBounder trait
  mod.rs          # UGraphic trait, UParam

src/svek/         # Graphviz 布局引擎（Java svek 包移植）
  builder.rs      # GraphvizImageBuilder（高层 API）
  mod.rs          # DotStringFactory（DOT 生成 + SVG 解析）
  node.rs         # SvekNode
  edge.rs         # SvekEdge（含 label 尺寸计算）
  svg_result.rs   # SVG 解析
  extremity.rs    # 箭头端点

src/skin/         # 主题系统
  rose.rs         # Rose 默认主题常量 + DrawOp
  arrow.rs        # ArrowConfiguration

src/style/        # CSS 样式系统
  mod.rs          # ISkinParam trait
  skin_param.rs   # 统一配置

src/abel/         # 统一数据模型
src/decoration/   # UML 装饰符号
src/dot/          # Graphviz DOT 语言工具
src/tim/          # 模板引擎
```

### Renderer 层（旧实现，手写 SVG）

```
src/render/svg.rs              # class/object 渲染 + 公共函数
src/render/svg_sequence.rs     # sequence 渲染
src/render/svg_component.rs    # component/deployment
src/render/svg_state.rs        # state diagram
src/render/svg_activity.rs     # activity diagram
src/render/svg_sprite.rs       # SVG sprite 嵌入
src/layout/graphviz.rs         # 旧 Graphviz 调用（class 在用）
src/layout/sequence.rs         # sequence 布局计算
```

---

## 核心问题与修复方向

### 最高原则

**以 Java PlantUML 实现为纲**。所有行为、属性顺序、数值格式、颜色、间距精确匹配 Java。

### 问题 1：底座模块孤立未接入（根因）

**现状**: renderer 全部手写 SVG (`write!(buf, "<rect ...")`），不用 SvgGraphic。
布局用旧的 `layout/graphviz.rs`，不用 svek。常量在 renderer 中重复定义。

**审计数据** (见 `docs/base_module_migration_audit.md`):
- 319 处手写 SVG 标签 → 应用 SvgGraphic
- 182 个硬编码颜色 → 应用 HColor / skin::rose
- 83 处直接调用 font_metrics → 应通过 StringBounder
- render/svg.rs 的 fmt_coord / xml_escape → 已委托 klimt（完成）
- NOTE_FOLD/BG/BORDER → 已统一到 skin::rose（完成）

**修复方向**: 自底向上逐步替换。不做大爆炸重构，一个 renderer 一个 renderer 迁移。

### 问题 2：Class diagram Graphviz 布局偏差

**现状**: `layout/graphviz.rs` 把 edge label 文本直接传给 Graphviz。
Java 用 `SvekLine` 计算 label 的像素尺寸，传给 Graphviz 的是固定大小占位节点。

**表现**: 每条 link 的 entity y 坐标偏移 ~2px，多条 link 累积。

**修复方向**: 让 class renderer 使用 `svek::builder::GraphvizImageBuilder`。
svek 底座已完整实现 Java 的 DOT 生成逻辑。
（注意：之前尝试迁移的 agent 在 layout/mod.rs 加了代码但造成回退，已回滚）

### 问题 3：Sequence diagram 宽度计算

**现状**: 参与者间距算法与 Java 不同。
- `needed = text_w + 24.0` 的 24px 是粗略估算
- note 位置计算导致总宽度偏大 (~21px)
- self-message 宽度计算有偏差

**修复方向**: 对照 Java 的 `SequenceDiagramFileMakerPuma2.java` 精确移植间距算法。

### 问题 4：Meta wrapping 高度

**现状**: 带 title/legend/footer/header/caption 的图高度差 6-9px。

**修复方向**: 对照 Java 的 `AnnotatedBuilder.java` 修正 block_dim / bordered_dim 计算。

### 问题 5：SvgGraphic API 缺口

**审计结果**:
- `<circle>` 元素: 已添加 `svg_circle()` (16 处需要)
- `text-anchor` 参数: 待添加到 `svg_text()` (30+ 处需要)
- `<tspan>` 支持: 缺 (12+ 处需要)
- `<image>`: 缺 (sprite)
- `<polyline>`: 缺 (gantt/ditaa)

---

## 已完成的迁移

1. `klimt::svg::fmt_coord()` — render/svg.rs 已委托
2. `klimt::svg::xml_escape()` — render/svg.rs 已委托
3. `skin::rose::NOTE_FOLD/BG/BORDER` — 7 个 renderer 已统一引用
4. `SvgGraphic::svg_circle()` — 已实现
5. `SvgGraphic` trace 日志 — svg_rectangle/svg_text/svg_line
6. sprite font-style oblique→italic — 已修复
7. sprite 3-digit hex → 6-digit — 已修复

---

## 下一步建议优先级

1. **迁移 class renderer 到 svek** — 谨慎渐进，先只替换布局数据源，保留渲染代码
2. **补齐 SvgGraphic API** — text-anchor, tspan
3. **逐个 renderer 替换手写 SVG** — 从最小的开始 (json → nwdiag → ditaa → ...)
4. **修复 sequence 间距算法** — 对照 Java 精确移植
5. **修复 meta wrapping** — title/legend/caption 高度计算

---

## 关键文件速查

| 文件 | 用途 |
|------|------|
| `docs/base_module_migration_audit.md` | 完整审计：319 处手写 SVG 详细清单 |
| `tests/reference_tests.rs` | reference test 框架（fuzzy numeric comparison） |
| `tests/reference/` | Java PlantUML 生成的标准 SVG |
| `tests/fixtures/` | 测试用 .puml 输入文件 |
| `src/klimt/svg.rs` | SvgGraphic 底座（Java 精确属性序） |
| `src/svek/builder.rs` | GraphvizImageBuilder（Java svek 移植） |
| `src/skin/rose.rs` | 统一常量 + DrawOp |
| `src/render/svg.rs:render_class_diagram()` | class 渲染入口 (~line 850) |
| `src/layout/sequence.rs:layout_sequence()` | sequence 布局入口 |

## 环境备注

- `/ext` 是独立 3.3T 盘，空间充足
- `/` 根分区仅 2.8G，Cargo target 在 `/ext` 下无问题
- `/tmp` 是独立 366G 盘
