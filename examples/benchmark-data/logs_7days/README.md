# 场景二测试数据：跨文件结构化查询

## 数据集说明

本数据集用于测试跨多个 CSV 日志文件的结构化查询能力。

## 数据规模

- **文件数量**: 168 个（7天 × 24小时）
- **每文件行数**: 50-100 条（含表头）
- **总日志行数**: ~12,600 条

## 数据结构（CSV 格式）

```
timestamp,level,module,message,request_id
2026-01-01T00:00:00Z,error,AuthService,Failed to authenticate user,req_12345
```

## 使用示例

### XORE 查询（单文件）

```bash
# 统计单个日志文件的级别分布
xore p examples/benchmark-data/logs_7days/app_01_00.csv \
  "SELECT level, COUNT(*) as count FROM app_01_00 GROUP BY level ORDER BY count DESC"

# 查找特定模块的错误
xore p examples/benchmark-data/logs_7days/app_01_00.csv \
  "SELECT timestamp, message FROM app_01_00 WHERE level = 'error' AND module = 'DatabasePool' LIMIT 10"
```

### 对比指标

| 指标 | XORE | DuckDB |
|------|------|--------|
| 查询耗时 | 待测试 | 待测试 |
| 内存占用 | 待测试 | 待测试 |
| Token 消耗 | 待测试 | N/A |

## 生成脚本

```bash
bash examples/benchmark-data/generate_logs_7days.sh
```
