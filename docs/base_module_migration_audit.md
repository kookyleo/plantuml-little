# 底座模块迁移审计

扫描时间: 2026-03-17
扫描范围: src/render/*.rs, src/layout/*.rs, src/parser/*.rs, src/lib.rs

---

## 汇总统计

| 模块                         | 待迁移数 | 影响文件数 |
|------------------------------|----------|------------|
| klimt::svg (SvgGraphic)      | 319      | 18         |
| klimt::color (HColor)        | 182      | 16         |
| klimt::font (StringBounder)  | 83       | 24         |
| skin::rose (常量/DrawOp)     | 64       | 14         |
| decoration::symbol (形状)    | 10       | 2          |
| klimt::svg (fmt_coord 重复)  | 2        | 2          |
| klimt::svg (xml_escape 重复) | 1        | 1          |
| dot 模块 (DOT 格式化)       | 1        | 1          |
| style::ISkinParam (配置接口) | 27       | 14         |
| abel::Entity/Link (数据模型) | 0        | 0          |
| svek::builder (布局编排)     | 0        | 0          |

---

## 按模块分类

### 1. klimt::svg — SvgGraphic (手写 SVG 元素)

**总数: 319 处 / 18 文件**

所有 render 文件直接用 `write!()` / `format!()` 拼接 SVG 标签字符串，而不是通过
SvgGraphic 的 `draw_rect()` / `draw_text()` / `draw_line()` 等方法生成。这是
影响面最大的一类。

#### svg_sequence.rs (69 处) — 最大

- :104 `<rect fill="#000000" ...` — lifeline 不可见占位矩形
- :113 `<line style="stroke:...` — lifeline 虚线
- :245 `<rect ...` — participant 框
- :260 `<text ...` — participant 名称文字
- :295 `<circle ...` — actor 头
- :304-340 多处 `<line ...` — actor 躯体/手/腿
- :352 `<text ...` — actor 名称
- :378 `<circle ...` — boundary/control/entity 图标
- :389-401 `<line ...` — boundary 横线/竖线
- :414-676 大量 `<text>`, `<path>`, `<ellipse>`, `<rect>` — 各种 participant 类型
- :807-819 `<text ...` — 消息编号
- :1033+ note 绘制: `<polygon>`, `<path>`, `<text>`
- :1102-1213 fragment/group: `<rect>`, `<path>`, `<line>`, `<text>`
- :1265-1310 divider: `<rect>`, `<text>`
- :1324-1352 ref frame: `<rect>`, `<path>`, `<text>`

#### svg_component.rs (43 处)

- :195-297 各类节点 (component/interface/node/package): `<rect>`, `<path>`, `<polygon>`, `<line>`, `<text>`
- :407-431 component 小盒子图标: `<rect>` ×4
- :452-512 database 节点: `<path>` (body + top ellipse)
- :530 cloud 节点: `<rect rx="20" ry="20">`
- :551-586 artifact/circle/box 节点: `<rect>`, `<circle>`, `<text>`
- :611-642 storage 节点: `<rect>`, `<polygon>`, `<line>`
- :667-746 folder/frame 节点: `<rect>`, `<path>`, `<line>`
- :777-795 agent/stack/queue: `<rect>`, `<text>`
- :1109+ note: `<polygon>`, `<path>`, `<line>`, `<text>`

#### svg.rs — class/object 图 (29 处)

- :282 `<rect ...` — 背景矩形
- :1126 `<path ...` — 箭头三角形
- :1243 `<rect ...` — class 框体
- :1288-1389 `<text ...` — 类名/stereotype/泛型文字
- :1350 `<ellipse ...` — 圆形字符图标
- :1473 `<rect ...` — generic type 框
- :1483 `<text ...` — generic type 文字
- :1518 `<rect ...` — object 框体
- :1547 `<text ...` — object 名称
- :1558 `<line ...` — 分隔线
- :1606 `<line ...` — 字段分隔线
- :1672 `<text ...` — 成员文字
- :1767 `<ellipse ...` — public 可见性图标
- :1779 `<rect ...` — private 可见性图标
- :1796 `<polygon ...` — protected 可见性图标
- :1816 `<polygon ...` — package 可见性图标
- :2241 `<polygon ...` — 箭头头部
- :2253 `<circle ...` — lollipop 接口图标
- :2276-2286 `<line ...` — 链接线段
- :2397 `<text ...` — 链接标签
- :2428 `<polygon ...` — note 形状
- :2451 `<path ...` — note fold
- :2484 `<line ...` — note 连接线

#### svg_state.rs (28 处)

- :174 `<polygon ...` — choice/fork 菱形
- :203 `<text ...` — transition 标签
- :285 `<text ...` — stereotype 文字
- :370 `<text ...` — state 名称
- :498 `<text ...` — transition 标签
- :602-612 `<text ...` — 描述文字
- 各种 `<rect>`, `<ellipse>`, `<circle>`, `<line>`, `<path>` — state 节点绘制

#### svg_erd.rs (24 处)

- entity/relationship/attribute/ISA 节点: `<rect>`, `<ellipse>`, `<polygon>`, `<line>`, `<text>`
- edge: `<path>`, `<polygon>`
- note: `<polygon>`, `<path>`, `<text>`

#### svg_activity.rs (21 处)

- start/stop/action/diamond/fork_bar: `<rect>`, `<ellipse>`, `<polygon>`, `<circle>`, `<text>`
- edge: `<path>`, `<polygon>`, `<line>`
- note: `<polygon>`, `<path>`, `<text>`
- swimlane: `<line>`, `<rect>`, `<text>`

#### svg_timing.rs (16 处)

- track/signal/concise: `<rect>`, `<polygon>`, `<line>`, `<text>`
- note: `<polygon>`, `<path>`, `<text>`

#### svg_usecase.rs (15 处)

- actor: `<circle>`, `<line>`, `<text>`
- use case: `<ellipse>`, `<text>`
- boundary: `<rect>`, `<line>`, `<text>`
- edge: `<line>`, `<polygon>`, `<text>`

#### svg_salt.rs (15 处)

- button/checkbox/radio/textfield/combobox: `<rect>`, `<circle>`, `<text>`, `<line>`
- tree node/table cell: `<rect>`, `<text>`

#### svg_wbs.rs (11 处)

- node: `<rect>`, `<text>`
- note: `<polygon>`, `<path>`, `<text>`

#### svg_sprite.rs (11 处)

- sprite 转路径: `<path>`, `<rect>`, `<text>`, `<circle>`

#### svg_gantt.rs (10 处)

- bar: `<rect>`, `<text>`
- note: `<polygon>`, `<path>`, `<text>`
- axis: `<line>`, `<text>`

#### svg_json.rs (8 处)

- :51 `<rect fill=...` — box 背景
- :65 `<text ...` — key 文字 (bold)
- :74 `<text ...` — value 文字
- :80 `<line ...` — 垂直分隔线
- :86 `<line ...` — 水平分隔线
- :92 `<rect fill="none" ...` — box 边框
- :99 `<path ...` — arrow 曲线路径
- :105 `<path ...` — arrow 三角头部

#### svg_richtext.rs (5 处)

- `<text>` / `<tspan>` creole 富文本

#### svg_mindmap.rs (5 处)

- :79 `<path ...` — edge 曲线
- :103 `<rect ...` — node 矩形
- :135 `<line ...` — note 连接虚线
- :148 `<polygon ...` — note 形状
- :160 `<path ...` — note fold

#### svg_ditaa.rs (4 处)

- `<rect>`, `<text>`, `<line>` — ditaa 文字图

#### svg_nwdiag.rs (3 处)

- `<rect>`, `<text>`, `<line>` — 网络拓扑

#### svg_hyperlink.rs (2 处)

- `<text>` — 超链接文字

---

### 2. klimt::color — HColor (手写颜色常量)

**总数: 约 182 个硬编码颜色常量 / 16 文件**

每个渲染器都在文件顶部定义自己的颜色常量字符串，而不是使用 HColor 或 Theme 系统。

#### svg_component.rs (18 个常量)

- :22-52 `COMPONENT_BG="#F1F1F1"`, `COMPONENT_BORDER="#181818"`, `RECT_BG`, `NODE_BG`, `DATABASE_BG`, `CLOUD_BG`, `EDGE_COLOR`, `TEXT_FILL`, `NOTE_BG="#FEFFDD"`, `NOTE_BORDER`, `GROUP_BG="#FFFFFF"`, `GROUP_BORDER`, `ARTIFACT_BG`, `STORAGE_BG`, `FOLDER_BG`, `FRAME_BG`, `AGENT_BG`, `STACK_BG`, `QUEUE_BG` 等

#### svg_sequence.rs (18 个常量)

- :22-38 `PARTICIPANT_BG="#E2E2F0"`, `PARTICIPANT_BORDER="#181818"`, `LIFELINE_COLOR`, `ARROW_COLOR`, `NOTE_BG="#FEFFDD"`, `NOTE_BORDER`, `GROUP_BG="#EEEEEE"`, `GROUP_BORDER="#000000"`, `ACTIVATION_BG="#FFFFFF"`, `ACTIVATION_BORDER`, `FRAGMENT_BG`, `FRAGMENT_BORDER`, `REF_BG`, `REF_BORDER`, `DESTROY_COLOR="#A80036"`, `DIVIDER_COLOR="#888888"`, `TEXT_COLOR`, `REF_FRAME_STROKE`, `REF_TAB_FILL`

#### svg.rs — class 图 (14 个常量)

- :129-165 `CLASS_BG="#F1F1F1"`, `CLASS_BORDER="#181818"`, `IFACE_BG`, `IFACE_BORDER`, `ENUM_BG`, `ENUM_BORDER`, `ABSTRACT_BG`, `ABSTRACT_BORDER`, `NOTE_BG="#FEFFDD"`, `NOTE_BORDER`, `LINK_COLOR="#181818"`, `LABEL_COLOR="#000000"`, `META_HF_COLOR="#888888"`, `LEGEND_BORDER_COLOR`, `LEGEND_BG`
- :1192-1196 内联: entity circle 颜色 `"#ADD1B2"`, `"#A9DCDF"`, `"#EB937F"` 等
- :1765-1816 内联: visibility icon 颜色 `"#84BE84"`, `"#F24D5C"`, `"#B38D22"`, `"#4177AF"` 及 stroke `"#038048"`, `"#C82930"`, `"#1963A0"`

#### svg_activity.rs (13 个常量)

- :21-34 `ACTION_BG="#F1F1F1"`, `ACTION_BORDER="#181818"`, `START_FILL="#222222"`, `STOP_FILL`, `DIAMOND_BG`, `DIAMOND_BORDER`, `FORK_FILL="#000000"`, `NOTE_BG`, `NOTE_BORDER`, `EDGE_COLOR`, `TEXT_FILL`, `SWIMLANE_BORDER`

#### svg_state.rs (9 个常量)

- :24-32 `STATE_BG="#F1F1F1"`, `STATE_BORDER="#181818"`, `INITIAL_FILL="#222222"`, `FINAL_OUTER="#000000"`, `FINAL_INNER`, `EDGE_COLOR`, `TEXT_FILL`, `NOTE_BG`, `NOTE_BORDER`

#### svg_erd.rs (12 个常量)

- :16-27 `ENTITY_BG`, `ENTITY_BORDER`, `RELATIONSHIP_BG`, `RELATIONSHIP_BORDER`, `ATTR_BG`, `ATTR_BORDER`, `EDGE_COLOR`, `TEXT_FILL`, `ISA_BG`, `ISA_BORDER`, `NOTE_BG`, `NOTE_BORDER`

#### svg_timing.rs (12 个常量)

- :19-32 `TRACK_BG_FILL`, `TRACK_BORDER`, `SIGNAL_STROKE`, `CONCISE_STROKE="#2E8B57"`, `ARROW_COLOR="#555555"`, `CONSTRAINT_COLOR="#FF6600"`, `TEXT_FILL`, `AXIS_LINE_COLOR="#888888"`, `AXIS_TEXT_COLOR="#333333"`, `TICK_COLOR="#CCCCCC"`, `NOTE_BG`, `NOTE_BORDER`

#### svg_gantt.rs (7 个常量)

- :20-28 `DEFAULT_BAR_FILL="#A4C2F4"`, `DEFAULT_BAR_STROKE="#3D85C6"`, `ARROW_COLOR="#555555"`, `TEXT_FILL`, `GRID_COLOR="#DDDDDD"`, `AXIS_TEXT_COLOR="#333333"`, `NOTE_BG`, `NOTE_BORDER`

#### svg_mindmap.rs (8 个常量)

- :17-25 `NODE_FILL="#F1F1F1"`, `ROOT_FILL="#FFD700"`, `NODE_BORDER`, `EDGE_COLOR`, `TEXT_COLOR`, `BORDER_WIDTH`, `NOTE_BG`, `NOTE_BORDER`

#### svg_wbs.rs (6 个常量)

- :15-21 `NODE_BG`, `NODE_BORDER`, `EDGE_COLOR`, `TEXT_FILL`, `NOTE_BG`, `NOTE_BORDER`

#### svg_json.rs (3 个常量)

- :14-16 `BOX_FILL="#F1F1F1"`, `BORDER_COLOR="#000000"`, `TEXT_COLOR="#000000"`

#### svg_nwdiag.rs (5 个常量)

- :14-19 `NETWORK_FILL="#F5F5F5"`, `NETWORK_BORDER="#A0A0A0"`, `SERVER_FILL`, `SERVER_BORDER`, `TEXT_FILL`, `CONNECTOR_COLOR="#888888"`

#### svg_usecase.rs (5 个常量)

- :19-24 `ACTOR_STROKE`, `UC_BG`, `UC_BORDER`, `BOUNDARY_BORDER="#444444"`, `EDGE_COLOR`, `TEXT_FILL`
- :243 内联 `"#F8F8FF"` — boundary 背景

#### svg_salt.rs (4 个常量)

- :12-15 `BG="#FFFFFF"`, `BORDER="#181818"`, `FILL="#F1F1F1"`, `TEXT="#000000"`
- :96 内联 `"#FFFFFF"` — 按钮背景
- :190 内联 `"#FFFFFF"` — checkbox 背景
- :225 内联 `"#FFFFFF"` — radio 背景

#### svg_ditaa.rs (4 个常量)

- :12-16 `BACKGROUND="#FFFFFF"`, `BOX_FILL="#F1F1F1"`, `BOX_BORDER="#333333"`, `TEXT_FILL`, `SHADOW_FILL`

#### layout/ditaa.rs (5 个内联)

- :216-220 `"#6666FF"`, `"#FF6666"`, `"#66CC66"`, `"#FFFF66"`, `"#EEEEEE"` — ditaa 颜色映射

#### layout/nwdiag.rs (1 个内联)

- :194 `"#E8F4FF"` — 网络高亮色

---

### 3. klimt::font — StringBounder (直接调用 font_metrics)

**总数: 约 83 处直接调用 / 24 文件**

所有文件直接 `use crate::font_metrics` 然后调用 `text_width()` / `line_height()` /
`ascent()` / `descent()`，而不是通过 `StringBounder` trait 或 `DefaultStringBounder`。

#### render/ 文件 (13 文件)

| 文件 | 调用次数 |
|------|----------|
| svg.rs | ~15 |
| svg_sequence.rs | ~20 |
| svg_component.rs | ~2 |
| svg_state.rs | ~7 |
| svg_activity.rs | ~2 |
| svg_usecase.rs | ~5 |
| svg_richtext.rs | ~8 |
| svg_salt.rs | ~1 |
| svg_json.rs | ~3 |
| svg_gantt.rs | ~1 |
| svg_wbs.rs | ~1 |
| svg_erd.rs | ~1 |
| svg_sprite.rs | ~1 |

#### layout/ 文件 (11 文件)

| 文件 | 调用次数 |
|------|----------|
| mod.rs | 多处 |
| sequence.rs | 多处 |
| component.rs | 多处 |
| activity.rs | 多处 |
| usecase.rs | 多处 |
| erd.rs | 多处 |
| gantt.rs | 多处 |
| timing.rs | 多处 |
| mindmap.rs | 多处 |
| wbs.rs | 多处 |
| salt.rs | 多处 |
| json_diagram.rs | 多处 |
| nwdiag.rs | 多处 |

---

### 4. skin::rose — 硬编码尺寸常量 (应统一到 rose 或 style)

**总数: 约 64 个重复/硬编码常量 / 14 文件**

以下常量在多个文件中重复定义，且与 `skin::rose` 已定义的常量含义重叠。

#### NOTE_FOLD 重复 (7 处)

- `src/render/svg.rs:140` — `NOTE_FOLD = 8.0`
- `src/render/svg_mindmap.rs:26` — `NOTE_FOLD = 8.0`
- `src/render/svg_erd.rs:28` — `NOTE_FOLD = 8.0`
- `src/render/svg_timing.rs:33` — `NOTE_FOLD = 8.0`
- `src/render/svg_gantt.rs:29` — `NOTE_FOLD = 8.0`
- `src/render/svg_wbs.rs:22` — `NOTE_FOLD = 8.0`
- `src/layout/sequence.rs:21` — `NOTE_FOLD = 10.0` (不同值!)

#### FONT_SIZE 重复 (10 处)

- `svg.rs:42` — 14.0
- `svg_json.rs:12` — 14.0
- `svg_component.rs:20` — 12.0
- `svg_sequence.rs:20` — 13.0
- `svg_activity.rs:18` — 13.0
- `svg_state.rs:19` — 13.0
- `svg_usecase.rs:18` — 12.0
- `svg_erd.rs:15` — 14.0
- `svg_mindmap.rs:15` — 12.0
- `svg_wbs.rs:12` — 12.0
- `svg_timing.rs:18` — 12.0
- `svg_gantt.rs:19` — 12.0
- `svg_ditaa.rs:10` — 12.0
- `svg_salt.rs` (内联) — 12.0

#### LINE_HEIGHT 重复 (8 处)

- `svg.rs:44` — 8.0
- `svg_component.rs:21` — 16.0
- `svg_sequence.rs:21` — 16.0
- `svg_activity.rs:19` — 16.0
- `svg_state.rs:22` — 16.0
- `svg_mindmap.rs:16` — 16.0
- `svg_ditaa.rs:11` — 16.0
- `svg_nwdiag.rs:13` — 16.0

#### 箭头/组件尺寸常量

skin::rose 已定义 ARROW_DELTA_X=10, ARROW_DELTA_Y=4, ARROW_PADDING_Y=4 等，
但 render 文件使用各自内联数值:

- `svg_sequence.rs:40` — `MARGIN = 5.0`
- `svg_sequence.rs:43-57` — fragment tab 几何常量 (FRAG_TAB_HEIGHT, FRAG_TAB_NOTCH 等)
- `svg.rs:48-163` — class 图 约 40 个微调常量
- `svg_usecase.rs:27-39` — actor 人形尺寸 (HEAD_R, BODY_LEN, ARM_SPREAD 等)

---

### 5. klimt::svg — fmt_coord / xml_escape 重复实现

**总数: 3 处 / 3 文件**

#### fmt_coord 重复

- `src/render/svg.rs:176` — `pub(crate) fn fmt_coord()` — 与 `klimt::svg::fmt_coord()` 逻辑完全相同
- `src/render/svg_sprite.rs:907` — `fn fmt_coord_raw()` — 类似功能的变体实现

所有 render 子模块通过 `use crate::render::svg::fmt_coord` 使用 render 版本，
而非 `klimt::svg::fmt_coord`。

#### xml_escape 重复

- `src/render/svg.rs:255` — `pub(crate) fn xml_escape()` — 与 `klimt::svg::xml_escape()` 逻辑完全相同

所有 render 子模块通过 `use crate::render::svg::xml_escape` 使用 render 版本。

---

### 6. decoration::symbol — 手绘 UML 符号形状

**总数: 约 10 处 / 2 文件**

`decoration::symbol` 已提供 `draw_database()`, `draw_cloud()`, `draw_folder()`,
`draw_node()`, `draw_storage()` 等返回 `SymbolShape` (含 UPath)。
但 `svg_component.rs` 和 `svg_sequence.rs` 中这些符号全部手写 SVG 路径。

- `svg_component.rs:391` — `render_component_node()` — 手写 component 矩形 + 小盒子
- `svg_component.rs:462` — `render_database_node()` — 手写 database 椭圆体 path
- `svg_component.rs:519` — `render_cloud_node()` — 手写 cloud 圆角矩形
- `svg_component.rs:595` — `render_artifact_node()` — 手写 artifact 折角
- `svg_component.rs:655` — `render_storage_node()` — 手写 storage 形状
- `svg_component.rs:678` — `render_folder_node()` — 手写 folder 标签 + 矩形
- `svg_component.rs:759` — `render_frame_node()` — 手写 frame 标签
- `svg_component.rs:806` — `render_agent_node()` — 手写 agent 矩形
- `svg_component.rs:827` — `render_stack_node()` — 手写 stack 多层矩形
- `svg_component.rs:872` — `render_queue_node()` — 手写 queue 形状

---

### 7. note 绘制重复

**总数: 10 处 / 10 文件**

几乎每个图类型渲染器都有独立的 `render_note()` 函数，逻辑高度相似（折角多边形 +
fold path + 文字），但没有共用 rose 的 `ComponentRoseNote` 或公共函数:

- `svg_sequence.rs:1033` — `draw_note()`
- `svg_component.rs:1109` — `render_note()`
- `svg_activity.rs:257` — `render_note()`
- `svg_state.rs:514` — `render_note()`
- `svg_erd.rs:445` — `render_note()`
- `svg_mindmap.rs:131` — `render_note()`
- `svg_timing.rs:456` — `render_note()`
- `svg_gantt.rs:228` — `render_note()`
- `svg_wbs.rs:188` — `render_note()`
- `svg.rs:2418+` — class note 绘制 (内联)

---

### 8. style::ISkinParam — 未对接统一配置

**总数: 约 27 处 / 14 文件**

所有渲染器使用简单的 `SkinParams` (HashMap wrapper from compat.rs)，而未接入
完整的 `style::ISkinParam` trait。`SkinParams.get_or()` 提供了基本的 skinparam
覆盖能力，但缺失:
- CSS style cascade 解析
- 按 element-level 查询 (e.g. `RoundCorner`, `Padding`)
- stereotype-aware 属性解析

受影响的文件 (每个 render 入口函数签名中都是 `skin: &SkinParams`):

- `src/render/svg.rs`
- `src/render/svg_sequence.rs`
- `src/render/svg_component.rs`
- `src/render/svg_activity.rs`
- `src/render/svg_state.rs`
- `src/render/svg_erd.rs`
- `src/render/svg_usecase.rs`
- `src/render/svg_mindmap.rs`
- `src/render/svg_timing.rs`
- `src/render/svg_gantt.rs`
- `src/render/svg_json.rs`
- `src/render/svg_salt.rs`
- `src/render/svg_wbs.rs`
- `src/render/svg_ditaa.rs`
- `src/render/svg_nwdiag.rs`

---

### 9. dot 模块 — 手写 DOT 格式化

**总数: 1 文件**

- `src/layout/graphviz.rs:118-164` — `fn to_dot()` 手工拼接 `digraph G { ... }` DOT 字符串

`dot::dot_data` 和 `dot::graphviz` 模块已提供更完善的 DOT 生成 API，但
`layout/graphviz.rs` 并未使用它们。

---

### 10. abel::Entity/Link — 统一数据模型

**总数: 0 — 完全未对接**

所有图类型使用各自的 model struct (`ClassDiagram`, `SequenceDiagram`, `ComponentDiagram` 等)，
`abel` 模块的 `Entity` / `Link` 统一数据模型在 render/layout 中未被使用。
这是架构层面的对齐任务，不属于简单的 API 替换。

---

### 11. svek::builder — Graphviz 布局编排

**总数: 0 — 完全未对接**

`svek` 模块 (builder, cluster, edge, node, snake, svg_result) 提供了高级的
Graphviz 布局编排 API，但 `layout/graphviz.rs` 使用的是独立的低级实现
(直接调用 `dot -Tsvg` 子进程 + SVG 解析)。

---

## 迁移优先级建议

1. **fmt_coord / xml_escape 去重** (3 处) — 最小改动，最大一致性收益
2. **颜色常量统一到 Theme/HColor** (182 处) — 消除每个文件的冗余定义
3. **NOTE_FOLD 等跨文件常量统一** (7+8+10 处) — 防止不一致
4. **note 绘制公共函数提取** (10 处) — 消除重复逻辑
5. **font_metrics 改为 StringBounder** (83 处) — 为测试 mock 打基础
6. **component 符号形状对接 decoration::symbol** (10 处) — 形状逻辑归位
7. **手写 SVG 迁移到 SvgGraphic** (319 处) — 最大工作量，核心架构改造
8. **SkinParams → ISkinParam** (27 处) — 依赖 style 模块完善度
9. **graphviz.rs → dot/svek 模块** (1 处) — 依赖 svek 模块完善度
10. **model → abel 统一数据模型** — 架构级重构，最后进行
