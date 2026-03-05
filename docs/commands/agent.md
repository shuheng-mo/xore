# agent 命令

Agent-Native 接口，为 AI Agent 提供高性能数据处理能力。

**别名:** `agent`

## 语法

```bash
xore agent [SUBCOMMAND] [OPTIONS]
xore agent --help
```

## 描述

`agent` 命令提供专为 AI Agent 设计的接口，通过**计算下推**和**结构化摘要**降低 90%+ Token 消耗。

**核心设计理念：**

- **计算下推**：将数据处理逻辑下推到 XORE 端执行，减少数据传输
- **结构化摘要**：通过 schema 和 sample 提供数据概览，避免全量数据传输
- **零拷贝**：使用 Polars 零拷贝读取，不加载完整文件到内存

## 子命令

| 子命令 | 功能 | 典型场景 |
|--------|------|----------|
| [`init`](#init-生成提示词模板) | 生成 Agent 提示词模板 | 初始化项目上下文 |
| [`schema`](#schema-获取数据结构) | 获取数据结构（零拷贝） | 了解数据列信息 |
| [`sample`](#sample-智能数据采样) | 智能数据采样 | 快速了解数据内容 |
| [`query`](#query-sql-查询) | SQL 查询 + JSON 输出 | 执行数据分析 |
| [`explain`](#explain-sql-错误分析) | SQL 错误分析 | 调试 SQL 问题 |

---

## init: 生成提示词模板

生成适用于不同 AI 模型的提示词模板。

### 语法

```bash
xore agent init [MODEL]
```

### 参数

| 参数 | 类型 | 必填 | 说明 |
|------|------|-----|------|
| `MODEL` | String | 否 | 模型名称（claude/gpt4/gemini），默认 claude |

### 示例

```bash
# 使用默认模型（Claude）
xore agent init

# 指定 Claude 模型
xore agent init claude

# 指定 GPT-4 模型
xore agent init gpt4

# 指定 Gemini 模型
xore agent init gemini
```

### 输出示例

```
# XORE Data Analysis Context

You are working with a dataset located at: {file_path}

## Available Tools

- `xore agent schema <file>` - Get data structure without reading full file
- `xore agent sample <file> <n>` - Get representative data samples
- `xore agent query <file> "<sql>"` - Execute SQL queries

## Data Schema

Use `xore agent schema` to understand the data structure first.
...
```

---

## schema: 获取数据结构

获取数据的 Schema 信息（列名、类型、统计信息），零拷贝实现，不读取完整文件。

### 语法

```bash
xore agent schema <FILE> [OPTIONS]
```

### 参数

| 参数 | 类型 | 必填 | 说明 |
|------|------|-----|------|
| `FILE` | String | 是 | 数据文件路径 |

### 选项

| 选项 | 短选项 | 类型 | 默认值 | 说明 |
|------|-------|------|-------|------|
| `--histogram` | - | bool | false | 显示列值分布直方图 |
| `--json` | - | bool | false | JSON 格式输出 |
| `--minify` | - | bool | false | 精简输出（仅列名和类型） |

### 示例

```bash
# 基本用法
xore agent schema data.csv

# JSON 格式输出
xore agent schema data.csv --json

# 显示列值分布
xore agent schema data.csv --histogram

# 精简模式
xore agent schema data.csv --minify
```

### 输出示例（文本格式）

```
Schema: data.csv
├── id: Int64
├── name: String
├── age: Int64
├── salary: Float64
└── department: String

Statistics:
├── id: unique=1000, nulls=0
├── name: unique=1000, nulls=0
├── age: min=22, max=65, mean=35.5, nulls=5
├── salary: min=3000, max=50000, mean=15000, nulls=10
└── department: unique=5, nulls=0
```

### 输出示例（JSON 格式）

```json
{
  "file": "data.csv",
  "columns": [
    {"name": "id", "dtype": "Int64", "unique": 1000, "nulls": 0},
    {"name": "name", "dtype": "String", "unique": 1000, "nulls": 0},
    {"name": "age", "dtype": "Int64", "min": 22, "max": 65, "mean": 35.5, "nulls": 5},
    {"name": "salary", "dtype": "Float64", "min": 3000, "max": 50000, "mean": 15000, "nulls": 10},
    {"name": "department", "dtype": "String", "unique": 5, "nulls": 0}
  ]
}
```

---

## sample: 智能数据采样

从数据集中采样代表性数据，支持多种采样策略。

### 语法

```bash
xore agent sample <FILE> <N> [OPTIONS]
```

### 参数

| 参数 | 类型 | 必填 | 说明 |
|------|------|-----|------|
| `FILE` | String | 是 | 数据文件路径 |
| `N` | usize | 是 | 采样行数 |

### 选项

| 选项 | 短选项 | 类型 | 默认值 | 说明 |
|------|-------|------|-------|------|
| `--strategy` | - | String | smart | 采样策略（random/head/tail/smart） |
| `--json` | - | bool | false | JSON 格式输出 |

### 采样策略

| 策略 | 说明 | 适用场景 |
|------|------|----------|
| `random` | 随机采样 | 快速查看数据 |
| `head` | 前 N 行 | 查看数据开头 |
| `tail` | 后 N 行 | 查看数据结尾 |
| `smart` | 分层采样 | **推荐**，保持数据分布代表性 |

### 示例

```bash
# 随机采样 100 行
xore agent sample data.csv 100 --strategy random

# 智能采样（推荐，保持数据分布）
xore agent sample data.csv 100 --strategy smart

# 前 100 行
xore agent sample data.csv 100 --strategy head

# 后 100 行
xore agent sample data.csv 100 --strategy tail

# JSON 格式输出
xore agent sample data.csv 100 --json
```

### 输出示例

```
Sample (smart, 100 rows from 10000):
┌──────┬─────────┬──────┬────────┬────────────┐
│ id   │ name    │ age  │ salary │ department │
├──────┼─────────┼──────┼────────┼────────────┤
│ 1    │ Alice   │ 28   │ 15000  │ Engineering│
│ 2    │ Bob     │ 35   │ 20000  │ Sales      │
│ 3    │ Charlie │ 42   │ 25000  │ Marketing  │
│ ...  │ ...     │ ...  │ ...    │ ...        │
└──────┴─────────┴──────┴────────┴────────────┘
```

---

## query: SQL 查询

执行 SQL 查询并以 JSON 格式返回结果。

### 语法

```bash
xore agent query <FILE> "<SQL>" [OPTIONS]
```

### 参数

| 参数 | 类型 | 必填 | 说明 |
|------|------|-----|------|
| `FILE` | String | 是 | 数据文件路径 |
| `SQL` | String | 是 | SQL 查询语句 |

### 选项

| 选项 | 短选项 | 类型 | 默认值 | 说明 |
|------|-------|------|-------|------|
| `--format` | - | String | json | 输出格式（json/csv） |

### 示例

```bash
# 基本查询
xore agent query data.csv "SELECT * FROM data LIMIT 10"

# 聚合查询
xore agent query data.csv "SELECT department, COUNT(*) as count, AVG(salary) as avg_salary FROM data GROUP BY department"

# 条件查询
xore agent query data.csv "SELECT * FROM data WHERE age > 30 AND salary > 10000"

# CSV 格式输出
xore agent query data.csv "SELECT * FROM data LIMIT 10" --format csv
```

### 输出示例（JSON）

```json
{
  "columns": ["id", "name", "age", "salary", "department"],
  "rows": [
    {"id": 1, "name": "Alice", "age": 28, "salary": 15000, "department": "Engineering"},
    {"id": 2, "name": "Bob", "age": 35, "salary": 20000, "department": "Sales"},
    {"id": 3, "name": "Charlie", "age": 42, "salary": 25000, "department": "Marketing"}
  ],
  "row_count": 3
}
```

---

## explain: SQL 错误分析

分析 SQL 错误并提供修复建议。

### 语法

```bash
xore agent explain "<SQL>"
```

### 参数

| 参数 | 类型 | 必填 | 说明 |
|------|------|-----|------|
| `SQL` | String | 是 | SQL 查询语句 |

### 示例

```bash
# 分析错误 SQL
xore agent explain "SELECT * FORM data"

# 分析复杂 SQL
xore agent explain "SELEC id, name FORM users WHER age > 18"
```

### 输出示例

```
SQL Error Analysis:
───────────────────
Original: SELECT * FORM data

Issues Found:
─────────────
1. [ERROR] Keyword typo: "FORM" should be "FROM"
   → Fix: Replace "FORM" with "FROM"

Corrected Query:
────────────────
SELECT * FROM data

Suggestion:
───────────
- Ensure table name "data" exists in the database
- Check column names if selecting specific columns
```

---

## 性能指标

| 指标 | 目标 | 实际 | 状态 |
|------|------|------|------|
| Schema 获取 | <10ms | ~5ms | ✅ |
| 100 行采样 | <50ms | ~30ms | ✅ |
| JSON 输出解析 | <100ms | ~80ms | ✅ |
| Token 节约比例 | >90% | ~95% | ✅ |

## 与 process 命令的区别

| 特性 | `xore process` | `xore agent` |
|------|---------------|--------------|
| 输出格式 | 表格/文本 | JSON |
| 目标用户 | 人类用户 | AI Agent |
| Schema 获取 | 读取完整文件 | 零拷贝 |
| 采样策略 | 固定 | 多种策略 |
| Token 消耗 | 高 | 低（>90% 节约） |

## 最佳实践

1. **先 Schema 后 Query**：先了解数据结构，再执行查询
2. **使用 Smart 采样**：保持数据分布代表性
3. **JSON 输出**：便于 AI 解析和处理
4. **错误使用 Explain**：快速定位和修复 SQL 问题
