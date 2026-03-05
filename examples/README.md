# XORE 示例

本目录包含 XORE 的使用示例、测试数据和性能基准测试脚本。

## 目录结构

```
examples/
├── benchmark-data/          # 测试数据
│   ├── small/              # 小型测试数据 (~10KB)
│   ├── medium/             # 中型测试数据 (~10MB)
│   ├── large/              # 大型测试数据 (~10GB)
│   └── README.md           # 数据说明
├── scripts/                 # 测试脚本
│   ├── benchmark_xore.sh   # XORE 性能测试
│   ├── compare_ripgrep.sh  # ripgrep 对比
│   ├── compare_duckdb.sh   # DuckDB 对比
│   └── compare_pandas.sh   # Pandas 对比
└── README.md               # 本文件
```

## 快速开始

### 1. 文件搜索

```bash
# 全文搜索
xore f "error" --path ./logs

# 文件类型过滤
xore f "database" --type csv

# 语义搜索
xore f --semantic "数据库连接失败"

# 增量搜索
xore f "TODO" --watch
```

### 2. 数据处理

```bash
# 数据预览
xore p sales.csv

# SQL查询
xore p sales.csv "SELECT category, SUM(total_amount) FROM this GROUP BY category"

# 数据质量检查
xore p sales.csv --quality-check

# 导出数据
xore p sales.csv "SELECT * FROM this WHERE status = 'completed'" -o output.csv
```

### 3. Agent 命令

```bash
# 初始化Agent
xore agent init claude

# 获取Schema
xore agent schema sales.csv --json

# 智能采样
xore agent sample sales.csv 100 --strategy smart

# SQL查询
xore agent query sales.csv "SELECT COUNT(*) FROM this"

# 错误分析
xore agent explain "SELECT * FORM sales"
```

### 4. 基准测试

```bash
# 完整测试
xore benchmark --suite all

# 扫描测试
xore benchmark --suite scan --data-path ./data

# I/O测试
xore benchmark --suite io --data-path ./data
```

## 测试数据

### 小型数据 (small/)

| 文件 | 格式 | 行数 | 用途 |
|------|------|------|------|
| sales_small.csv | CSV | 100 | 基本查询测试 |
| sales_small.json | JSON | 100 | JSON解析测试 |
| users_small.csv | CSV | 50 | 用户数据测试 |
| server_log_small.log | LOG | 500 | 日志搜索测试 |
| server_log_small.json | JSON | 100 | JSON日志测试 |
| access_log.tsv | TSV | 1000 | TSV格式测试 |
| error_log.yaml | YAML | 50 | YAML格式测试 |
| test_small.db | SQLite | 100 | SQLite测试 |

### 中型数据 (medium/)

| 文件 | 格式 | 行数 | 用途 |
|------|------|------|------|
| sales_medium.csv | CSV | 100K | 中等规模处理 |
| users_medium.csv | CSV | 50K | 用户数据处理 |
| server_log_medium.log | LOG | 200K | 中型日志处理 |
| test_medium.db | SQLite | 10K | SQLite测试 |

### 大型数据 (large/)

| 文件 | 格式 | 大小 | 用途 |
|------|------|------|------|
| data_*.csv | CSV | ~10GB | 大文件性能测试 |

## 性能基准

运行性能测试：

```bash
# XORE 性能测试
bash scripts/benchmark_xore.sh

# 竞品对比测试
bash scripts/compare_ripgrep.sh
bash scripts/compare_duckdb.sh
bash scripts/compare_pandas.sh
```

## 依赖

- XORE (已安装)
- ripgrep (可选): `brew install ripgrep`
- DuckDB (可选): `brew install duckdb`
- Python 3.8+ (可选): 用于生成测试数据

## 文档

- [命令文档](../docs/commands/)
- [配置文档](../docs/reference/)
- [开发笔记](../supplementary/dev-notes.md)
