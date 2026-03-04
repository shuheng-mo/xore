# process 命令

数据处理与分析命令，基于 **Polars** 高性能数据引擎。

**别名:** `p`

## 语法

```bash
xore process [OPTIONS] <FILE> [QUERY]
xore p [OPTIONS] <FILE> [QUERY]
```

## 描述

`process` 命令用于处理和分析数据文件，采用 Polars 引擎提供高性能数据处理能力。支持：

- **数据预览**：表格格式显示前 10 行数据
- **数据质量检查**：缺失值、重复行、列统计、离群值检测
- **SQL 查询**：基于 Polars SQL 引擎，支持完整 SQL 语法 ✅
- **零拷贝读取**：大文件（>1MB）自动使用 `memmap2` 内存映射
- **惰性求值**：`LazyFrame` 模式优化内存占用，支持超大数据集

## 参数

### 位置参数

| 参数 | 类型 | 必填 | 说明 |
|-----|------|-----|------|
| `FILE` | String | 是 | 数据文件路径 |
| `QUERY` | String | 否 | SQL 查询语句 |

### 选项参数

| 选项 | 类型 | 默认值 | 说明 |
|-----|------|-------|------|
| `--quality-check` | bool | false | 执行数据质量检查 |
| `-o, --output` | String | - | 输出文件路径（支持 csv, json, parquet 格式）|
| `-f, --format` | String | - | 导出格式（如不指定，从输出文件扩展名推断）|

## 支持的文件格式

### 输入格式

| 格式 | 扩展名 | 预览 | 质量检查 | SQL 查询 |
|-----|-------|-----|---------|---------|
| CSV | .csv | ✅ | ✅ | ✅ |
| JSON | .json | ✅ | ✅ | ❌ |
| Parquet | .parquet | ✅ | ✅ | ✅ |

### 导出格式

| 格式 | 扩展名 | 支持 | 压缩 | 流式导出 |
|-----|-------|-----|------|---------|
| CSV | .csv | ✅ | ❌ | ✅ |
| JSON | .json, .jsonl | ✅ | ❌ | ✅ |
| Parquet | .parquet | ✅ | ✅ | ✅ |
| Arrow IPC | .arrow | ✅ | ❌ | ✅ |

**性能特性：**

- CSV/Parquet 文件 >1MB 自动启用零拷贝内存映射（`memmap2`）
- 自动 Schema 推断（默认扫描前 1000 行）
- LazyFrame 惰性执行，延迟计算优化内存

## 使用示例

### 数据预览

```bash
# 预览 CSV 文件（显示前 10 行）
xore process data.csv
xore p data.csv

# 预览 Parquet 文件
xore p data.parquet

# 预览 JSON 文件
xore p config.json
```

### 数据质量检查

```bash
# 检查 CSV 数据质量
xore process data.csv --quality-check
xore p sales.csv --quality-check

# 检查 Parquet 数据质量
xore p metrics.parquet --quality-check

# 检查 JSON 数据质量
xore p users.json --quality-check
```

### SQL 查询

```bash
# 基本查询
xore p data.csv "SELECT * FROM data WHERE age > 30"
xore p users.csv "SELECT name, email FROM users WHERE active = true"

# 聚合查询
xore p sales.csv "SELECT category, SUM(price * quantity) as revenue FROM sales GROUP BY category"
xore p metrics.csv "SELECT region, AVG(score) as avg_score FROM metrics GROUP BY region ORDER BY avg_score DESC"

# 多表 JOIN（需要先注册多个表，当前版本支持单文件查询）
xore p orders.csv "SELECT * FROM orders WHERE total > 1000 LIMIT 10"

# 复杂查询示例
xore p sales.csv "SELECT
    category,
    COUNT(*) as count,
    SUM(revenue) as total_revenue,
    AVG(revenue) as avg_revenue
FROM sales
WHERE date >= '2024-01-01'
GROUP BY category
ORDER BY total_revenue DESC
LIMIT 5"
```

**注意事项：**

- 表名使用文件名（去除扩展名），例如 `data.csv` 的表名为 `data`
- 支持标准 SQL 语法：`SELECT`, `WHERE`, `GROUP BY`, `ORDER BY`, `LIMIT`, `JOIN`
- 支持聚合函数：`COUNT`, `SUM`, `AVG`, `MIN`, `MAX`
- 结果最多显示 100 行（避免终端溢出）

## 输出示例

### CSV 预览

```
📄 数据预览: data.csv

 id  | name    | age | city
-----|---------|-----|----------
 1   | Alice   | 28  | Beijing
 2   | Bob     | 32  | Shanghai
 3   | Charlie | 25  | Shenzhen

显示前 3 行 (共 1,000 行)
```

### JSON 预览（数组）

```
📄 数据预览: users.json

 id  | name    | email
-----|---------|------------------
 1   | Alice   | alice@example.com
 2   | Bob     | bob@example.com

数组包含 100 个元素，显示前 10 个
```

### JSON 预览（对象）

```
📄 数据预览: config.json

键                | 值
------------------|------------------
database.host     | localhost
database.port     | 5432
server.timeout    | 30

对象包含 15 个字段，显示前 20 个
```

### 数据质量检查报告

```
🔍 数据质量检查: data.csv

基本信息
  ✓ 总行数: 1,000
  ✓ 总列数: 4

发现的问题
  ⚠ 发现 1 列存在缺失值
    - age: 5.2% 缺失 (52 行)
  ⚠ 检测到 15 行重复数据

建议
  💡 运行 'xore p data.csv --deduplicate' 去除重复行
  💡 检查数据源，确保必填字段有值
```

## 质量检查项

### CSV/Parquet 质量检查（基于 Polars）

| 检查项 | 说明 | 实现状态 |
|-------|------|---------|
| 行数统计 | 总行数 | ✅ |
| 列数统计 | 总列数 | ✅ |
| 数据类型 | 自动推断的列类型 | ✅ |
| 缺失值检测 | 按列统计空值数量和百分比 | ✅ |
| 重复行检测 | 完全相同的行数（使用 Polars `is_duplicated`）| ✅ |
| 列统计信息 | 唯一值数量、缺失值百分比 | ✅（API 可用）|
| 离群值检测 | IQR 方法检测数值列异常值 | ✅（API 可用）|

### JSON 质量检查

| 检查项 | 说明 | 实现状态 |
|-------|------|---------|
| 格式一致性 | JSON 结构异常检测 | ✅ |
| 字段统计 | 对象字段数量、数组元素数量 | ✅ |
| 结构验证 | 数组元素字段一致性检查 | ✅ |

**注：** CLI 当前输出缺失值和重复行统计，更多统计项可通过 API 调用 `DataProfiler` 获取。

## 数据导出功能 ✨

### 导出查询结果

```bash
# 导出为 CSV
xore p data.csv "SELECT * FROM data WHERE age > 30" -o output.csv

# 导出为 JSON (JSONL 格式)
xore p sales.csv "SELECT category, SUM(revenue) as total FROM sales GROUP BY category" -o report.json

# 导出为 Parquet（列式存储，高压缩比）
xore p large.csv "SELECT * FROM large WHERE status = 'active'" -o filtered.parquet

# 指定导出格式（覆盖扩展名推断）
xore p data.csv "SELECT * FROM data" -o output.txt -f csv
```

### 导出特性

- **自动格式检测**：从输出文件扩展名推断格式
- **流式导出**：支持 GB 级数据导出，内存占用 <100MB
- **压缩支持**：Parquet 格式支持 Gzip/Zstd 压缩
- **高性能**：使用 Polars 引擎，导出速度快

### 导出示例输出

```
⚙️ 执行 SQL 查询...
文件: sales.csv
查询: SELECT category, SUM(revenue) as total FROM sales GROUP BY category

💾 导出数据到 report.csv...
✓ 导出完成 (150 行, 2 列, 3,245 字节)
```

## 性能优化

### SIMD 数值计算 ✨

- 使用循环展开优化的数值计算
- 性能提升 2-3x（求和、均值、方差等）
- 支持大规模数据统计分析

### 零拷贝与惰性求值

- 文件 >1MB 自动使用 `memmap2` 内存映射
- 使用 Polars `LazyFrame` 延迟执行
- Polars 自动并行化查询，利用多核 CPU

## 相关命令

- [find](./find.md) - 查找数据文件
- [benchmark](./benchmark.md) - 测试处理性能

## 另请参阅

- [配置文件参考](../reference/configuration.md)
