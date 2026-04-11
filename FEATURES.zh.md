# plantuml-little 功能支持

对齐 Java PlantUML **v1.2026.2** (`bb8550d`)。

## 图表类型 — 29 种完整实现 + 5 种文本/透传 + 3 种不支持

### 完整实现（与 Java 逐字节一致）

| 类型 | 起始标签 | 布局引擎 | Ref 测试数 |
|------|----------|----------|-----------|
| Class（类图） | `@startuml` | Graphviz (Smetana) | 14 |
| Sequence（序列图） | `@startuml` | 内置引擎 (Puma / Teoz) | 32 |
| Activity v3（活动图） | `@startuml` | 内置引擎 | 11 |
| State（状态图） | `@startuml` | Graphviz | 13 |
| Component / Deployment（组件 / 部署图） | `@startuml` | Graphviz | 11 |
| Use Case（用例图） | `@startuml` | Graphviz | 3 |
| Object（对象图） | `@startuml` | Graphviz | 4 |
| Timing（时序图） | `@startuml` | 内置引擎 | 2 |
| ERD (Chen)（ER 图） | `@startchen` | Graphviz | 6 |
| Gantt（甘特图） | `@startgantt` | 内置引擎 | 1 |
| JSON | `@startjson` | 内置引擎 | 1 |
| YAML | `@startyaml` | 内置引擎 | 1 |
| Mindmap（思维导图） | `@startmindmap` | 内置引擎 | 1 |
| WBS（工作分解） | `@startwbs` | 内置引擎 | 5 |
| NWDiag（网络图） | `@startnwdiag` | 内置引擎 | 1 |
| Salt / Wireframe（线框图） | `@startsalt` | 内置引擎 | 1 |
| DOT (Graphviz) | `@startdot` | 子进程透传 | 1 |
| EBNF | `@startebnf` | 内置引擎 | 2 |
| Regex（正则可视化） | `@startregex` | 内置引擎 | 3 |
| BPM（业务流程） | `@startbpm` | 内置引擎 | 4 |
| Board（看板） | `@startboard` | 内置引擎 | 1 |
| Chronology（年表） | `@startchronology` | 内置引擎 | 1 |
| Chart（图表） | `@startchart` | 内置引擎 | 2 |
| Pie（饼图） | `@startpie` | 内置引擎 | 1 |
| HCL | `@starthcl` | 内置引擎 | 1 |
| Flow（流程图） | `@startflow` | 内置引擎 | 2 |
| Wire（接线图） | `@startwire` | 内置引擎 | 2 |
| Archimate（架构图） | `@startuml` | Graphviz | 2 |
| Packet（报文结构） | `@startpacket` | 内置引擎 | 1 |

### 文本 / 透传类型

| 类型 | 说明 | Ref 测试数 |
|------|------|-----------|
| Creole | `@startcreole` — 富文本标记渲染 | 1 |
| Def | `@startdef` — 纯文本显示 | 1 |
| Math | `@startmath` — 公式占位（Java 需外部工具） | 1 |
| LaTeX | `@startlatex` — 公式占位（Java 需外部工具） | 1 |
| Git | `@startgit` — Git 日志可视化 | 2 |
| Files | `@startfiles` — 文件树展示 | 2 |

### 明确不支持

| 类型 | 起始标签 | 原因 |
|------|----------|------|
| DITAA | `@startditaa` | Java 委托给第三方光栅化器（无 SVG 模式），从零实现 ASCII art → SVG 不在范围内 |
| JCCKIT | `@startjcckit` | Java AWT 专属图表库，仅 `Graphics2D` 输出，无 Rust 对等实现 |
| Project | `@startproject` | Java stable v1.2026.2 自身亦不支持此类型 |

## 预处理器

完整的预处理器管道，在解析前展开所有指令。

### 变量与赋值
- `!$var = value` — 变量赋值（Str / Int / Array 三种类型）
- `?=` 条件赋值
- `!local` 局部变量
- `!undef` 取消定义

### 条件
- `!if` / `!ifdef` / `!ifndef` / `!else` / `!elseif` / `!endif`
- 布尔逻辑：`&&`, `||`, `!`, 括号分组

### 函数与过程
- `!function` / `!endfunction`
- `!procedure` / `!endprocedure`
- `!unquoted procedure`
- `!return` 支持表达式求值
- 参数默认值
- `%call_user_func()` / `%invoke_procedure()` 动态调用

### 宏
- `!define NAME body`
- `!define NAME(params) body`
- `!definelong NAME` … `!enddefinelong`

### 循环
- `!foreach $var in collection` … `!endfor`
- `!while condition` … `!endwhile`（10,000 次上限防护）
- 嵌套循环

### 文件包含
- `!include path` — 本地相对路径
- `!include <stdlib/module>` — 内置标准库
- `!include http://...` / `!includeurl` — 远程 URL
- `!include_once` / `!include_many`
- `!includesub file!PART` — 子段选取
- `!import archive.zip` — ZIP/JAR 归档导入

### 主题
- `!theme NAME` — 内置主题
- `!theme NAME from local/dir`
- `!theme NAME from <subdir>`
- `!theme NAME from https://...`

### 内置函数（35+）

`%strlen`, `%substr`, `%strpos`, `%splitstr`, `%splitstr_regex`, `%string`,
`%lower`, `%upper`, `%chr`, `%ord`, `%newline`, `%breakline`,
`%intval`, `%boolval`, `%not`, `%mod`, `%dec2hex`, `%hex2dec`,
`%size`, `%true`, `%false`,
`%variable_exists`, `%function_exists`,
`%get_variable_value`, `%set_variable_value`,
`%filename`, `%dirpath`, `%file_exists`, `%getenv`,
`%get_all_theme`, `%get_all_stdlib`

### 其他
- `!pragma key value`
- `!assert condition`
- `!dump_memory`（兼容 stub）
- 行续连（尾部 `\`）
- 算术表达式求值（+−×÷%，运算符优先级，括号）

## 样式系统

### skinparam
- 30+ 属性：BackgroundColor, FontColor, FontSize, FontName, BorderColor, ArrowColor, RoundCorner 等
- 元素级别覆盖：`skinparam classFontColor`, `skinparam sequenceArrowColor` 等
- 颜色规范化：`#RGB` → `#RRGGBB`，命名色，`transparent`
- 渐变支持：`#color1|color2`、`#color1/color2`
- 全部图表类型均已接入

### 方向
- `left to right direction` / `top to bottom direction`
- 支持 Class, Sequence, Activity, State, Component, ERD, WBS

### 主题
- 内置 rose 默认主题（30 色域字段）
- SkinParams 自动回退到主题默认值

## 富文本 / Creole 标记

### 行内格式
- `**bold**` / `<b>bold</b>`
- `//italic//` / `<i>italic</i>`
- `__underline__` / `<u>underline</u>`
- `~~strike~~` / `<s>strike</s>`
- `""monospace""`
- `<color:red>text</color>`
- `<size:18>text</size>`
- `<back:yellow>text</back>`
- `<font:courier>text</font>`
- `<sub>subscript</sub>` / `<sup>superscript</sup>`
- `~` 转义字符

### 块级元素
- `* item` — 无序列表
- `# item` — 有序列表
- `|= H | H |` / `| v | v |` — 表格
- `----` — 水平线

### 链接
- `[[url]]`
- `[[url label]]`
- `[[url{tooltip} label]]`

### 图片与图标
- `<img:path>` — 嵌入图片引用
- `<&icon>` — OpenIconic 图标（223 个内置图标）
- `<$sprite>` — 自定义 SVG sprite 引用

## SVG Sprite

- `sprite name <svg>...</svg>` — 单行/多行 SVG 定义
- `sprite $name <svg>...</svg>` — $ 前缀可选
- `<$name>` — 文本中引用 sprite
- viewBox 感知缩放，内联嵌入为 `<g>` 元素
- 支持复杂 SVG 特性：渐变、变换、文本样式、嵌入图像

## 序列图特性

### 参与者形状
`participant`, `actor`, `boundary`, `control`, `entity`, `database`, `collections`, `queue`

### 组合片段
`alt/else`, `loop`, `opt`, `par`, `break`, `critical`, `group`, `ref over`

### 其他
- 分隔符 `==...==`
- 延迟 `...`
- `autonumber [start]`
- 参与者颜色
- 手绘模式

## 活动图特性

### 控制流
- if / else / elseif 分支
- while / repeat-while 循环
- fork / join 并行
- goto / label 跳转
- break 退出
- backward 反向循环

### 泳道
- `|Swimlane|` 语法
- 多泳道并排渲染
- 跨泳道 L 型边路由

## 状态图特性

### 伪状态
- Fork / Join 横条
- Choice 菱形
- History `[H]` / Deep History `[H*]`

### 并发域
- `--` 分隔符

## 元数据

- `title` / `title ... end title`
- `header` / `footer`
- `legend` / `legend ... end legend`
- `caption`

## 跨图表功能

- Note 渲染：折角多边形 + 虚线连接器（全部图表类型）
- 超链接 / tooltip
- 手绘模式（`skinparam handwritten true`），与 Java 的 jiggle RNG 一致
- 渐变填充（线性 `|` / 径向 `/`）
- 源码种子 SVG ID（确定性输出）
- 错误处理：行号/列号定位，错误页面生成
- CJK / Unicode 字符宽度计算
- 多块 PUML 渲染
- 嵌入子图支持

## 输出格式

- **SVG** — 唯一输出格式

## 不在范围内

- PNG / PDF / EPS / ASCII 等其他输出格式
- GUI / Web Server / FTP / Pipe 模式
- PlantUML Server URL 编解码
- 安全沙箱
- ELK 布局引擎
- 完整 plantuml-stdlib（仅按需 vendor）
- 完整上游主题目录

## 测试覆盖

| 类别 | 数量 |
|------|------|
| 单元测试 | 2,693 |
| 集成测试 | 185 |
| Reference 测试（逐字节） | 337 |
| 忽略（不支持） | 3 |
| **总计** | **3,215** |
