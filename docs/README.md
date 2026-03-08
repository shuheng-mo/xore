# XORE 文档

> Explore the Abyss, Extract the Core

XORE 是一个高性能的本地 CLI 工具，将语义搜索与即时数据分析融为一体，且针对开发者和智能体的使用习惯进行了原生优化，让数据探索变得前所未有的简单和高效。

## 快速链接

- [快速入门](./getting-started.md) - 安装和基本用法
- [命令参考](./commands/README.md) - 所有命令详细说明
- [技术参考](./reference/filters.md) - 过滤器、配置、环境变量

## 安装

### 从源码编译

```bash
# 克隆仓库
git clone https://github.com/yourusername/xore.git
cd xore

# 编译 release 版本
cargo build --release

# 可执行文件位于
./target/release/xore --help
```

### 使用 Cargo 安装

```bash
cargo install xore
```

## 核心功能

| 功能 | 命令 | 说明 |
|-----|------|------|
| 文件搜索 | `xore find` | 高性能文件扫描与内容搜索 |
| 全文索引 | `xore find --index` | 基于 Tantivy 的中英文全文搜索 |
| 增量监控 | `xore find --index --watch` | 实时文件监控与增量索引更新 |
| 搜索历史 | `xore find --history` | 自动记录搜索历史，智能推荐 |
| 数据处理 | `xore process` | CSV/JSON/Parquet 数据预览与质量检查 |
| **SQL 查询** | `xore process <file> "<sql>"` | **基于 Polars SQL 引擎的完整 SQL 支持** ✅ |
| Polars 引擎 | `xore process` | 零拷贝读取、LazyFrame 惰性求值 |
| 数据质量分析 | `xore process --quality-check` | 缺失值、重复行、离群值检测 |
| **Agent 接口** | `xore agent` | **Agent-Native 接口，降低 90%+ Token 消耗** 🚀 |
| 性能测试 | `xore benchmark` | 系统性能基准测试 |
| 内存优化 | - | mimalloc 高性能分配器集成 |

## 文档结构

```
docs/
├── README.md              # 本文件 - 文档索引
├── getting-started.md     # 快速入门指南
├── semantic-search-guide.md # 语义搜索指南
├── commands/              # 命令参考
│   ├── README.md          # 命令概览
│   ├── find.md            # find 命令详解
│   ├── process.md         # process 命令详解
│   ├── agent.md          # agent 命令详解
│   └── benchmark.md       # benchmark 命令详解
└── reference/             # 技术参考
    ├── filters.md         # 过滤器语法参考
    ├── configuration.md   # 配置文件参考
    └── environment.md     # 环境变量参考
```

## 版本信息

- Rust 最低版本: 1.70+
- 支持平台: Linux, macOS, Windows
- 测试覆盖: 215+ 个单元测试 + 4 个集成测试全部通过 ✅
- 代码质量: cargo fmt + clippy + check 通过 ✅
- 测试覆盖率: >80%
- **Agent-Native 定位**：通过计算下推和结构化摘要降低 90%+ Token 消耗

## 最新功能

### 文档与示例完善 ✅

XORE MVPb版本已完成，文档和示例也已完善：

- Core Framework（CLI、文件扫描、日志、错误处理）
- Search Engine（Tantivy、增量索引、搜索优化）
- Data Reactor（Polars、SQL、SIMD、导出）
- AI & Polishing（ONNX、语义搜索、Agent、文档）

### SQL 查询引擎 ✅

基于 Polars `SQLContext` 的完整 SQL 支持，让数据分析更加灵活：

```bash
# 基本查询
xore p sales.csv "SELECT * FROM sales WHERE price > 100"

# 聚合分析
xore p sales.csv "SELECT category, SUM(price * quantity) as revenue
                  FROM sales GROUP BY category ORDER BY revenue DESC"

# 多表 JOIN
xore p users.csv "SELECT users.name, SUM(orders.amount) as total
                  FROM users INNER JOIN orders ON users.id = orders.user_id
                  GROUP BY users.name"
```

**支持的 SQL 功能：**

- ✅ SELECT, WHERE, GROUP BY, ORDER BY, LIMIT
- ✅ 聚合函数：COUNT, SUM, AVG, MIN, MAX
- ✅ 多表 JOIN：INNER JOIN, LEFT JOIN
- ✅ 复杂表达式和子查询

### Agent-Native 接口 🚀

XORE 提供专为 AI Agent 设计的接口，通过**计算下推**和**结构化摘要**降低 90%+ Token 消耗：

```bash
# 初始化 Agent 提示词
xore agent init claude

# 获取数据结构（零拷贝）
xore agent schema data.csv --json

# 智能数据采样
xore agent sample data.csv 100 --strategy smart

# SQL 查询（JSON 输出）
xore agent query data.csv "SELECT * FROM data LIMIT 10"

# SQL 错误分析
xore agent explain "SELECT * FORM data"
```

**Agent 命令优势：**

- ✅ 零拷贝 Schema 获取（不读取完整文件）
- ✅ 智能采样保持数据分布代表性
- ✅ JSON 结构化输出便于 AI 解析
- ✅ 多模型提示词模板（Claude/GPT-4/Gemini）
- ✅ **Token 节约 >90%**

## 获取帮助

```bash
# 查看帮助
xore --help

# 查看特定命令帮助
xore find --help
xore process --help
xore agent --help
xore benchmark --help
```
