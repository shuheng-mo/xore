#!/bin/bash
# =============================================================================
# 场景二测试数据生成脚本：跨文件结构化查询
# 
# 背景：智能体需要从数百个 CSV 日志文件中聚合统计信息
# 对比对象：DuckDB、pandas
# 注意：xore p 的 SQL 查询支持 CSV 和 Parquet 格式
# =============================================================================

set -e

OUTPUT_DIR="examples/benchmark-data/logs_7days"
MODULE_COUNT=8
FILES_PER_DAY=24

# 模块列表（模拟真实应用场景）
MODULES=(
    "AuthService"
    "DatabasePool"
    "APIGateway"
    "CacheManager"
    "MessageQueue"
    "Scheduler"
    "FileStorage"
    "NotificationService"
)

# 日志级别及权重 (整数表示，总和 100)
LEVELS=("error" "warning" "info" "debug")
LEVEL_WEIGHTS=(5 15 60 20)

# 错误消息模板
ERROR_MESSAGES=(
    "Failed to authenticate user"
    "Database connection timeout"
    "Invalid API key provided"
    "Rate limit exceeded"
    "Session expired"
    "Permission denied"
    "Resource not found"
    "Internal server error"
)

# 警告消息模板
WARNING_MESSAGES=(
    "Connection pool near capacity"
    "High memory usage detected"
    "Slow query detected"
    "Cache miss rate increasing"
    "Certificate expiring soon"
    "Disk space low"
    "Retry attempt failed"
    "Deprecated API called"
)

# 信息消息模板
INFO_MESSAGES=(
    "Request processed successfully"
    "User logged in"
    "Cache updated"
    "Job completed"
    "Data synchronized"
    "Health check passed"
    "Configuration reloaded"
    "Metrics collected"
)

echo "=========================================="
echo "场景二：跨文件结构化查询 - 测试数据生成"
echo "=========================================="

# 清理旧数据
echo "[1/4] 清理旧数据..."
rm -rf "$OUTPUT_DIR"
mkdir -p "$OUTPUT_DIR"

# 生成 CSV 日志文件（xore p SQL 查询支持 CSV 格式）
echo "[2/4] 生成 CSV 日志文件..."

for day in {1..7}; do
    for hour in $(seq 0 23); do
        filename="$OUTPUT_DIR/app_$(printf "%02d" $day)_$(printf "%02d" $hour).csv"
        
        # 每个文件生成 50-100 条日志
        line_count=$((50 + RANDOM % 51))
        
        # 写入 CSV 表头
        echo "timestamp,level,module,message,request_id" > "$filename"
        
        for ((i=0; i<line_count; i++)); do
            # 随机选择模块
            module_idx=$((RANDOM % MODULE_COUNT))
            module="${MODULES[$module_idx]}"
            
            # 根据权重随机选择日志级别
            rand=$((RANDOM % 100))
            cumulative=0
            level="info"
            message=""
            
            for idx in "${!LEVELS[@]}"; do
                cumulative=$((cumulative + LEVEL_WEIGHTS[$idx]))
                if [ $rand -lt $cumulative ]; then
                    level="${LEVELS[$idx]}"
                    break
                fi
            done
            
            # 根据级别选择消息
            case $level in
                error)
                    msg_idx=$((RANDOM % ${#ERROR_MESSAGES[@]}))
                    message="${ERROR_MESSAGES[$msg_idx]}"
                    ;;
                warning)
                    msg_idx=$((RANDOM % ${#WARNING_MESSAGES[@]}))
                    message="${WARNING_MESSAGES[$msg_idx]}"
                    ;;
                *)
                    msg_idx=$((RANDOM % ${#INFO_MESSAGES[@]}))
                    message="${INFO_MESSAGES[$msg_idx]}"
                    ;;
            esac
            
            # 生成 CSV 行
            timestamp="2026-01-$(printf "%02d" $day)T$(printf "%02d" $hour):$((RANDOM % 60)):$((RANDOM % 60))Z"
            request_id="req_$RANDOM$RANDOM"
            
            echo "$timestamp,$level,$module,$message,$request_id" >> "$filename"
        done
    done
done

# 生成统计信息
echo "[3/4] 生成统计信息..."

file_count=$(find "$OUTPUT_DIR" -name "*.csv" | wc -l)
error_count=$(grep -rh ",error," "$OUTPUT_DIR"/*.csv 2>/dev/null | wc -l)
warning_count=$(grep -rh ",warning," "$OUTPUT_DIR"/*.csv 2>/dev/null | wc -l)

echo "  - 文件数量: $file_count"
echo "  - Error 级别: $error_count"
echo "  - Warning 级别: $warning_count"

# 生成 README 说明
echo "[4/4] 生成说明文档..."

cat > "$OUTPUT_DIR/README.md" << 'EOF'
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
EOF

echo ""
echo "=========================================="
echo "测试数据生成完成！"
echo "=========================================="
echo ""
echo "数据位置: $OUTPUT_DIR"
echo "文件数量: $file_count"
echo "Error 日志: $error_count"
echo ""
echo "运行查询测试（单文件）："
echo "  xore p $OUTPUT_DIR/app_01_00.csv \"SELECT level, COUNT(*) as count FROM app_01_00 GROUP BY level\""
echo ""
