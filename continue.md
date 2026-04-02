# Continue: Architecture Alignment Plan

## Current: 260/296 (87.8%)

## 7 Architectural Gaps — 23/36 remaining failures

### Gap 1: State Composite — Single-Graph Cluster Model (5 tests)

**Java 架构**: ONE graphviz call per diagram. Composite states are `subgraph cluster_X { ... }` containing their children as real nodes. Graphviz handles cross-composite ranking naturally.

**Rust 现状**: TWO-LEVEL solve. Inner children get a separate `layout_children_with_graphviz()` call, then the composite is a single rect node in the outer solve. This prevents graphviz from ranking across composite boundaries.

**差距根因**:
- `state_history001` (+50px): `Paused → Active[H]` back-edge can't influence rank because `Active[H]` isn't in the outer graph
- `scxml0004` (+74px width): `Somp` inner cluster width differs because it's pre-computed, not graphviz-determined
- `scxml0003` (1px): render_dy rounding

**修复方案**:
1. 在 `layout_state()` 中，把 composite children 展平为 outer DOT 的 cluster subgraph nodes
2. 使用 `LayoutClusterSpec` 的 `sub_clusters` 字段实现递归嵌套
3. 保留 `a/p0/main/i/p1` 5-level wrapping（已在 component 中实现，commit `dba0672`）
4. Inner solve 退化为仅计算 composite header overhead

**涉及文件**: `src/layout/state.rs` (compute_state_node, layout_state), `src/svek/mod.rs` (cluster solve)
**估算**: 2 sessions

---

### Gap 2: Sequence Classic — freeY Frontier Model (5 tests)

**Java 架构**: `DrawableSetInitializer` 维护 `freeY` 作为 Y 轴前沿。每个 component 的 `getPreferredHeight()` 推进 freeY。Note 直接放在 freeY 处。

**Rust 现状**: `y_cursor` 跟踪箭头 Y 位置，note 用 `back_offset` 从 `msg_y` 反推。Sprite 的 preferred height（含 sprite 图像高度）不反映到 y_cursor 推进量。

**差距根因**:
- `testGradientSprite`/`testPolylineSprites` (-64px): sprite 消息后的 note 被放在 msg_y - back_offset 处，但 back_offset 抵消了 sprite 额外高度，导致 note 位置和 Java 差 64px
- `svgFillColourTest_2174`: sprite-in-stereotype 导致宽度计算异常
- `deployment_mono_multi`: multi-line body 中的 `<code>` 块高度

**修复方案**:
1. 引入 `free_y: f64` 独立于 `y_cursor`，按 Java 的 `freeY.add(comp.getPreferredHeight())` 推进
2. Note 位置改用 `free_y` 而非 `msg_y - back_offset`
3. Sprite preferred height = max(text_h, sprite_h) + 2*paddingY + arrowDeltaY

**涉及文件**: `src/layout/sequence.rs` (layout_sequence, note positioning)
**估算**: 1 session

---

### Gap 3: Usecase — Svek Pipeline (1 test)

**Java 架构**: Usecase 走 `CucaDiagramFileMakerSvek` → `GraphvizImageBuilder` → graphviz solve。Actors 是 `EntityImageActor`（stick figure UPath），use cases 是 `EntityImageUseCase`（ellipse）。

**Rust 现状**: 自定义 `layout_usecase()` 手动定位，不走 graphviz。Actor 用 circle+lines，不是 UPath。

**修复方案**:
1. 让 usecase 走 `layout_with_svek` pipeline（像 class/component 一样）
2. Actor 节点用 `ShapeType::Circle` 或新增 `Actor` shape
3. Use case 节点用 `ShapeType::Oval`

**涉及文件**: `src/layout/usecase.rs`, `src/render/svg_component.rs` (或新增 svg_usecase.rs)
**估算**: 1 session

---

### Gap 4: Subdiagram `{{ }}` Embedding (3 tests)

**Java 架构**: `SubjectDiagramFactory` 解析 `{{ }}` 块，递归调用 `UmlDiagramFactory` 生成内部图表的 `TextBlockDrawable`，嵌入父图。

**Rust 现状**: 完全未实现。`{{ }}` 被忽略。

**修复方案**:
1. Parser: 在 component/class parser 中识别 `{{ }}` 块，提取内部 puml 源码
2. Layout: 递归调用 `convert()` 生成内部 SVG
3. Render: 将内部 SVG 嵌入为 `<g transform="translate(x,y)">` 子图

**涉及文件**: `src/parser/component.rs`, `src/lib.rs` (convert 递归入口), `src/render/svg_component.rs`
**估算**: 2 sessions

---

### Gap 5: Teoz — GroupingTile Recursive Model (4 tests)

**Java 架构**: `GroupingTile` 递归包含子 tiles。`getPreferredHeight()` = header + Σ children.preferredHeight + footer。`ElseTile` 作为分隔符 tile 参与高度计算。

**Rust 现状**: 扁平 tile 列表，`FragmentStart/End/Separator` 作为标记。高度线性累加，不递归。

**差距根因**:
- `TT_0007` (-38px): `?` participant 的 fragment 高度未正确递归
- `TT_0009` (+128px width): 嵌套 group 的 extent 计算和 width 互相影响

**修复方案**:
1. 将 `TeozTile` 的 `FragmentStart/End` 替换为 `GroupingTile { children: Vec<TeozTile>, ... }`
2. `preferred_height()` 递归求和子 tile
3. Extent 计算递归进入 GroupingTile 内部

**涉及文件**: `src/layout/sequence_teoz/builder.rs`
**估算**: 2 sessions

---

### Gap 6: Component Note — Graphviz-Routed Positioning (3 tests)

**Java 架构**: Notes 是 graphviz 节点，通过 invisible edge 连接到目标 entity。Graphviz 决定 note 位置。Note shape 用 Opale path（含 ear connector）。

**Rust 现状**: Notes 手动定位在 entity 旁边（上/下/左/右），不参与 graphviz solve。最近加了 state note 的 graphviz edge 定位，但 component 还是手动的。

**修复方案**:
1. 在 `layout_component()` 中把 notes 加为 graphviz 节点
2. 用 invisible edge 连接 note → target entity
3. 从 graphviz 结果获取 note 位置
4. Component note 用 Opale path 渲染（已实现部分，commit `f0ac83b`）

**涉及文件**: `src/layout/component.rs`, `src/render/svg_component.rs`
**估算**: 1 session

---

### Gap 7: C4 Stdlib Macro System (2 tests)

**Java 架构**: `!include <C4/C4_Container>` 从内置 stdlib 加载宏定义。C4 宏（`Person()`, `Container()`, `System_Boundary()`）展开为标准 PlantUML 元素。

**Rust 现状**: `!include` 支持本地文件，但 `<...>` stdlib 路径未解析。C4 宏定义不可用。

**修复方案**:
1. 方案 A: 内嵌 C4 stdlib 宏定义（~500 行 puml）
2. 方案 B: 实现 `!include <path>` 从 bundled stdlib 目录加载
3. C4 宏展开后就是标准 component 图——不需要新渲染引擎

**涉及文件**: `src/preproc/mod.rs` (include resolver), 新增 `src/stdlib/` 目录
**估算**: 1 session

---

## 执行优先级

| 优先级 | Gap | Tests | 理由 |
|--------|-----|-------|------|
| **P1** | Gap 2: Sequence freeY | 5 | 模型清晰，影响范围大，1 session |
| **P2** | Gap 6: Component Notes | 3 | 基础设施已有，扩展到 component |
| **P3** | Gap 1: State Cluster | 5 | component 的 cluster 经验可复用 |
| **P4** | Gap 5: Teoz GroupingTile | 4 | 递归模型替换，中等复杂度 |
| **P5** | Gap 7: C4 Stdlib | 2 | 纯数据工作，不涉及引擎 |
| **P6** | Gap 3: Usecase Svek | 1 | pipeline 已有，接入即可 |
| **P7** | Gap 4: Subdiagram | 3 | 递归渲染，最复杂 |

**预估总工作量: 10 sessions → 23 tests → 283/296 (95.6%)**

剩余 13 tests 是独立引擎特性（timing/gantt/legacy-activity/handwritten/Chen-ordering/mindmap-balance/class-note/link-tooltip）。
