# plantuml-little Feature Support

本文档记录项目当前的功能覆盖范围，供使用者参考和后续维护跟踪。

## Diagram Types — 17 种

| 类型 | 起始标记 | 布局引擎 | Fixture 数量 |
|------|----------|----------|-------------|
| Class | `@startuml` | Graphviz | 14 |
| Sequence | `@startuml` | 自有引擎 | 31 |
| Activity v3 | `@startuml` | 自有引擎 | 8 |
| State | `@startuml` | Graphviz | 13 |
| Component / Deployment | `@startuml` | Graphviz | 10 |
| Use Case | `@startuml` | Graphviz | 3 |
| Object | `@startuml` | Graphviz | (复用 Class) |
| Timing | `@startuml` | 自有引擎 | 2 |
| ERD (Chen) | `@startchen` | Graphviz | 5 |
| Gantt | `@startgantt` | 自有引擎 | 1 |
| JSON | `@startjson` | 自有引擎 | 1 |
| YAML | `@startyaml` | 自有引擎 | 1 |
| Mindmap | `@startmindmap` | 自有引擎 | 1 |
| WBS | `@startwbs` | 自有引擎 | 5 |
| DITAA | `@startditaa` | 自有引擎 | 1 |
| NWDiag | `@startnwdiag` | 自有引擎 | 1 |
| Salt / Wireframe | `@startsalt` | 自有引擎 | 1 |
| DOT (Graphviz) | `@startdot` | 子进程透传 | 1 |

## Preprocessor

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
- `!while condition` … `!endwhile`（10000 次上限防护）
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

### 内置函数 (35+)

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

## Style System

### skinparam
- 30+ 属性：BackgroundColor, FontColor, FontSize, FontName, BorderColor, ArrowColor, RoundCorner 等
- 元素级别覆盖：`skinparam classFontColor`, `skinparam sequenceArrowColor` 等
- 颜色规范化：`#RGB` → `#RRGGBB`，命名色，`transparent`
- 全部 17 种图表类型均已接入

### Direction
- `left to right direction` / `top to bottom direction`
- 支持 Class, Sequence, Activity, State, Component, ERD, WBS

### Theme
- 内置 rose 默认主题（30 色域字段）
- SkinParams 自动回退到主题默认值

## Rich Text / Creole Markup

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

## SVG Sprite

- `sprite name <svg>...</svg>` — 单行/多行 SVG 定义
- `sprite $name <svg>...</svg>` — $ 前缀可选
- `<$name>` — 文本中引用 sprite
- viewBox 感知缩放，内联嵌入为 `<g>` 元素
- 支持复杂 SVG 特性：渐变、变换、文本样式、嵌入图像

## Sequence Diagram 扩展

### 参与者形状
`participant`, `actor`, `boundary`, `control`, `entity`, `database`, `collections`, `queue`

### 组合片段
`alt/else`, `loop`, `opt`, `par`, `break`, `critical`, `group`, `ref over`

### 其他
- `divider ==...==`
- `delay ...`
- `autonumber [start]`
- 参与者颜色

## State Diagram 扩展

### 伪状态
- Fork / Join 横条
- Choice 菱形
- History `[H]` / Deep History `[H*]`

### 并发域
- `--` 分隔符

## Activity Diagram 扩展

### 泳道
- `|Swimlane|` 语法
- 多泳道并排渲染
- 跨泳道 L 型边路由

## Metadata

- `title` / `title ... end title`
- `header` / `footer`
- `legend` / `legend ... end legend`
- `caption`

## 跨图表功能

- Note 渲染：折角多边形 + 虚线连接器（全部图表类型）
- 超链接 / tooltip
- 错误处理：行号 + 列号定位
- CJK / Unicode 字符宽度计算
- SVG 输出验证

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
| Unit Tests | 1,319 |
| Integration Tests | 183 |
| Test Fixtures | 296 |
| **Total** | **1,502** |
