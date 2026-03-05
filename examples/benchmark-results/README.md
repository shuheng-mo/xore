# XORE 性能基准测试报告

**测试时间:** 2026-03-05
**测试环境:** macOS ARM64 (Apple Silicon)
**XORE 版本:** 1.0.0 (Rust)

---

## 1. 测试环境

| 工具 | 版本 |
|------|------|
| XORE | 1.0.0 (Rust) |
| ripgrep | 15.1.0 |
| DuckDB | 1.4.4 |
| Python | 3.14.3 |
| Pandas | 3.0.1 |
| PyArrow | 23.0.1 |

---

## 2. 测试数据

### 2.1 小型数据集 (~10KB)

| 文件 | 格式 | 行数 | 大小 |
|------|------|------|------|
| sales_small.csv | CSV | 100 | 8.5KB |
| sales_small.json | JSON | 100 | 29KB |
| sales_small.parquet | Parquet | 100 | ~3KB |
| users_small.csv | CSV | 50 | 4.2KB |
| server_log_small.log | LOG | 500 | 27KB |
| access_log.tsv | TSV | 1000 | 45KB |
| error_log.yaml | YAML | 50 | 3.6KB |
| test_small.db | SQLite | 100 | 16KB |

### 2.2 中型数据集 (~10MB)

| 文件 | 格式 | 行数 | 大小 |
|------|------|------|------|
| sales_medium.csv | CSV | 100,000 | 8.6MB |
| sales_medium.parquet | Parquet | 100,000 | ~2MB |
| users_medium.csv | CSV | 50,000 | 4.4MB |
| server_log_medium.log | LOG | 200,000 | 11.3MB |
| test_medium.db | SQLite | 10,000 | 792KB |

### 2.3 大型数据集 (~600MB per file)

| 文件 | 格式 | 行数 | 大小 |
|------|------|------|------|
| data_1.csv ~ data_17.csv | CSV | ~10M/文件 | ~600MB/文件 |

---

## 3. 实际测试结果

### 3.1 文件搜索性能测试

| 场景 | ripgrep | XORE | 胜者 | 性能提升 |
|------|---------|------|------|----------|
| 小型日志 (500行, 27KB) | 14ms | 60ms | ripgrep | - |
| 中型日志 (200K行, 11MB) | 13ms | **10ms** | **XORE** | **1.3x** |

### 3.2 小型数据处理测试 (100行)

| 格式 | DuckDB | XORE | 胜者 | 性能提升 |
|------|--------|------|------|----------|
| CSV COUNT | 82ms | **30ms** | **XORE** | **2.7x** |
| JSON COUNT | N/A | **13ms** | XORE | - |
| Parquet COUNT | 26ms | **26ms** | 平手 | - |

### 3.3 中型数据处理测试 (100K行)

| 场景 | DuckDB | XORE | 胜者 | 性能提升 |
|------|--------|------|------|----------|
| CSV COUNT | 433ms | **43ms** | **XORE** | **10x** |
| CSV GROUP BY | 156ms | **34ms** | **XORE** | **4.6x** |
| CSV WHERE | 126ms | **26ms** | **XORE** | **4.8x** |
| Parquet COUNT | 21ms | **9ms** | **XORE** | **2.3x** |

### 3.4 大型数据处理测试 (600MB CSV, ~10M行)

| 工具 | COUNT 查询 | 性能对比 |
|------|------------|----------|
| DuckDB | 501ms | 基准 |
| Pandas | 8060ms | 慢 16x |
| XORE | 1268ms | 慢 2.5x |

**结论:** 大型文件处理需要优化

---

## 4. 性能对比汇总表

### 4.1 文件搜索

| 场景 | ripgrep | XORE | 胜者 |
|------|---------|------|------|
| 小型日志 | 14ms | 60ms | ripgrep |
| 中型日志 | 13ms | **10ms** | **XORE** |

### 4.2 数据处理 - 小型数据集

| 场景 | DuckDB | XORE | 胜者 |
|------|--------|------|------|
| CSV COUNT | 82ms | **30ms** | XORE |
| JSON COUNT | N/A | **13ms** | XORE |
| Parquet COUNT | 26ms | 26ms | 平手 |

### 4.3 数据处理 - 中型数据集

| 场景 | DuckDB | XORE | 胜者 |
|------|--------|------|------|
| CSV COUNT | 433ms | **43ms** | XORE |
| CSV GROUP BY | 156ms | **34ms** | XORE |
| CSV WHERE | 126ms | **26ms** | XORE |
| Parquet COUNT | 21ms | **9ms** | XORE |

### 4.4 数据处理 - 大型数据集 (600MB)

| 场景 | DuckDB | Pandas | XORE | 胜者 |
|------|--------|--------|------|------|
| CSV COUNT | 501ms | 8060ms | 1268ms | DuckDB |

---

## 5. 分析与结论

### 5.1 XORE 优势

1. **中型数据处理**: 比 DuckDB 快 4-10 倍
2. **Parquet 格式**: 性能显著优于 DuckDB
3. **统一接口**: 支持 CSV、JSON、Parquet 多种格式

### 5.2 待优化场景

1. **小型文件搜索**: 首次搜索有冷启动开销
2. **大型文件处理**: 需要优化大型 CSV 文件的读取性能

### 5.3 总体评价

XORE 在中等规模数据处理场景下性能表现优异：

- 中型数据处理显著快于 DuckDB (4-10x)
- Parquet 格式处理性能最佳
- 大型文件处理需要进一步优化

---

## 6. 运行测试

```bash
# 运行完整性能测试
bash examples/benchmark-results/run_benchmark.sh

# 查看最新测试结果
ls -lt examples/benchmark-results/benchmark_results_*.txt | head -1
```

---

**报告生成时间:** 2026-03-05
**测试方法:** 每次测试运行 3 次取平均值
