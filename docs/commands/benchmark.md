# benchmark 命令

性能基准测试命令。

**别名:** `bench`

## 语法

```bash
xore benchmark [OPTIONS]
xore bench [OPTIONS]
```

## 描述

`benchmark` 命令用于测试 XORE 各组件的性能。支持：
- 文件扫描性能测试
- I/O 吞吐量测试
- 内存分配性能测试
- 多种输出格式

## 参数

| 选项 | 短选项 | 类型 | 默认值 | 说明 |
|------|--------|------|--------|------|
| `--suite` | `-s` | enum | `all` | 测试套件 |
| `--output` | `-o` | enum | `text` | 输出格式 |
| `--iterations` | `-n` | usize | `3` | 迭代次数 |
| `--data-path` | - | String | `.` | 测试数据路径 |
| `--warmup` | - | usize | `1` | 预热次数 |

## 测试套件

| 套件 | 说明 | 状态 |
|------|------|------|
| `all` | 运行所有测试（默认）| ✅ |
| `scan` | 文件扫描性能 | ✅ |
| `io` | I/O 吞吐量 | ✅ |
| `alloc` | 内存分配性能 | ✅ |
| `search` | 全文搜索性能 | 🔄 开发中 |
| `process` | 数据处理性能 | 🔄 开发中 |

## 输出格式

| 格式 | 说明 |
|------|------|
| `text` | 人类可读的文本格式（默认）|
| `json` | JSON 格式，便于程序解析 |
| `csv` | CSV 格式，便于导入表格软件 |

## 使用示例

### 基本用法

```bash
# 运行所有基准测试
xore benchmark
xore bench

# 指定测试套件
xore benchmark --suite scan
xore benchmark --suite io
xore benchmark --suite alloc
```

### 自定义参数

```bash
# 增加迭代次数获得更准确的结果
xore benchmark --iterations 10

# 增加预热次数
xore benchmark --warmup 3 --iterations 5

# 指定测试数据路径
xore benchmark --suite scan --data-path /data
```

### 输出格式

```bash
# 输出 JSON 格式
xore benchmark --output json > results.json

# 输出 CSV 格式
xore benchmark --output csv > results.csv

# 组合使用
xore benchmark --suite scan -n 5 --output json
```

## 输出示例

### 文本格式

```
XORE 性能基准测试 (分配器: mimalloc)

测试路径: ., 迭代次数: 3, 预热: 1

测试结果
────────────────────────────────────────────────────────────
✓ 文件扫描 (1,234 文件, 56 目录): 45.3ms (27,242 files/s)
✓ 目录遍历 (深度 10): 12.1ms
✓ 顺序读取 (10.5 MB): 23.1ms (455 MB/s)
✓ 顺序写入 (1 MB): 8.7ms (114 MB/s)
✓ Vec<String> 分配 (100K 元素): 3.9ms (25,752,593 allocs/s)
✓ HashMap<String, usize> (50K 条目): 2.6ms (19,029,191 ops/s)
✓ 小字符串分配/释放 (50K 次): 1.8ms (28,108,300 allocs/s)
⏳ 全文搜索: 待实现
⏳ 数据处理: 待实现
```

### JSON 格式

```json
[
  {
    "name": "文件扫描 (1,234 文件, 56 目录)",
    "duration_ms": 45.3,
    "throughput": "27,242 files/s",
    "status": "success"
  },
  {
    "name": "顺序读取 (10.5 MB)",
    "duration_ms": 23.1,
    "throughput": "455 MB/s",
    "status": "success"
  }
]
```

### CSV 格式

```csv
name,duration_ms,throughput,status
"文件扫描 (1,234 文件, 56 目录)",45.3,27242 files/s,success
"顺序读取 (10.5 MB)",23.1,455 MB/s,success
```

## 测试说明

### 扫描测试 (scan)

| 测试项 | 说明 |
|--------|------|
| 文件扫描 | 扫描指定目录下所有文件，统计吞吐量 |
| 目录遍历 | 深度 10 的目录遍历性能 |

### I/O 测试 (io)

| 测试项 | 说明 |
|--------|------|
| 顺序读取 | 读取文件的吞吐量 (MB/s) |
| 顺序写入 | 写入 1MB 数据的吞吐量 (MB/s) |

### 分配测试 (alloc)

| 测试项 | 说明 |
|--------|------|
| Vec<String> 分配 | 分配 100K 个字符串的速度 |
| HashMap 操作 | 50K 次插入操作的速度 |
| 小字符串分配 | 50K 次分配/释放周期的速度 |

## 内存分配器

XORE 默认使用 `mimalloc` 高性能内存分配器。基准测试会显示当前使用的分配器。

编译时选择分配器：
```bash
# 使用 mimalloc（默认）
cargo build --release

# 使用系统分配器
cargo build --release --no-default-features
```

## 相关命令

- [find](./find.md) - 实际文件搜索
- [process](./process.md) - 实际数据处理

## 另请参阅

- [环境变量参考](../reference/environment.md) - 线程数配置
