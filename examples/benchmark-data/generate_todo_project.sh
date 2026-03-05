#!/bin/bash
# =============================================================================
# 场景三测试数据生成脚本：智能体迭代调试场景
# 
# 背景：模拟智能体多轮检索完成"找出项目中所有未处理的 TODO"
# 对比对象：ripgrep + 手动读文件
# =============================================================================

set -e

PROJECT_DIR="examples/benchmark-data/todo_project"

# TODO 消息池
TODO_MESSAGES=(
    "Implement user authentication flow"
    "Add error handling for database connection"
    "Optimize query performance"
    "Add unit tests for this module"
    "Refactor legacy code"
    "Update API documentation"
    "Add caching layer"
    "Implement retry logic"
    "Fix memory leak in connection pool"
    "Add input validation"
    "Implement rate limiting"
    "Add logging for debugging"
    "Update dependencies"
    "Add support for pagination"
    "Implement search functionality"
)

# FIXME 消息池
FIXME_MESSAGES=(
    "Memory leak in connection pool"
    "Race condition in cache update"
    "Buffer overflow in string handling"
    "Thread safety issue"
    "Performance regression"
    "Incorrect error handling"
    "Data loss in edge case"
)

# 模块列表
MODULES=(
    "auth"
    "database"
    "api"
    "utils"
    "models"
    "services"
    "middleware"
    "handlers"
)

echo "=========================================="
echo "场景三：智能体迭代调试 - 测试数据生成"
echo "=========================================="

# 清理旧数据
echo "[1/5] 清理旧数据..."
rm -rf "$PROJECT_DIR"
mkdir -p "$PROJECT_DIR/src"

# 创建目录结构
for module in "${MODULES[@]}"; do
    mkdir -p "$PROJECT_DIR/src/$module"
    mkdir -p "$PROJECT_DIR/tests/$module"
    mkdir -p "$PROJECT_DIR/docs/$module"
done

# 生成源文件
echo "[2/5] 生成源文件..."

file_count=0
for module in "${MODULES[@]}"; do
    # 每个模块生成 3-8 个源文件
    file_num=$((3 + RANDOM % 6))
    
    for ((i=1; i<=file_num; i++)); do
        filename="$PROJECT_DIR/src/$module/${module}_${i}.rs"
        
        # 随机生成 2-5 个 TODO
        todo_count=$((2 + RANDOM % 4))
        
        {
            echo "// Module: $module"
            echo "// Description: Core functionality for $module"
            echo ""
            echo "use std::collections::HashMap;"
            echo "use crate::error::XoreError;"
            echo ""
            
            # 添加 TODO
            for ((t=0; t<todo_count; t++)); do
                msg_idx=$((RANDOM % ${#TODO_MESSAGES[@]}))
                echo "// TODO: ${TODO_MESSAGES[$msg_idx]}"
            done
            
            # 偶尔添加 FIXME
            if [ $((RANDOM % 3)) -eq 0 ]; then
                fixme_idx=$((RANDOM % ${#FIXME_MESSAGES[@]}))
                echo "// FIXME: ${FIXME_MESSAGES[$fixme_idx]}"
            fi
            
            echo ""
            echo "pub struct Manager {"
            echo "    config: Config,"
            echo "    cache: HashMap<String, String>,"
            echo "}"
            echo ""
            echo "impl Manager {"
            echo "    pub fn new(config: Config) -> Self {"
            echo "        Self { config, cache: HashMap::new() }"
            echo "    }"
            echo ""
            echo "    pub fn initialize(&mut self) -> Result<(), XoreError> {"
            echo "        // Implementation"
            echo "        Ok(())"
            echo "    }"
            echo "}"
        } > "$filename"
        
        file_count=$((file_count + 1))
    done
done

# 生成测试文件
echo "[3/5] 生成测试文件..."

for module in "${MODULES[@]}"; do
    test_num=$((1 + RANDOM % 3))
    
    for ((i=1; i<=test_num; i++)); do
        filename="$PROJECT_DIR/tests/$module/test_${module}_${i}.rs"
        
        {
            echo "#[cfg(test)]"
            echo "mod tests {"
            echo "    use super::*;"
            echo ""
            
            # 添加 TODO
            todo_count=$((1 + RANDOM % 2))
            for ((t=0; t<todo_count; t++)); do
                msg_idx=$((RANDOM % ${#TODO_MESSAGES[@]}))
                echo "    // TODO: ${TODO_MESSAGES[$msg_idx]}"
            done
            
            echo ""
            echo "    #[test]"
            echo "    fn test_basic() {"
            echo "        assert!(true);"
            echo "    }"
            echo "}"
        } > "$filename"
    done
done

# 生成配置文件（应被忽略）
echo "[4/5] 生成配置文件..."

cat > "$PROJECT_DIR/Cargo.toml" << 'EOF'
[package]
name = "todo-project"
version = "0.1.0"
edition = "2021"

# TODO: Update dependencies
# TODO: Add more features
EOF

cat > "$PROJECT_DIR/config.yaml" << 'EOF'
# Configuration
# TODO: Add environment-specific config
# TODO: Add validation

database:
  host: localhost
  port: 5432
EOF

cat > "$PROJECT_DIR/README.md" << 'EOF'
# Todo Project

A sample project with TODO comments for testing.

## TODO List
- [ ] Complete implementation
- [ ] Add tests
- [ ] Update documentation
EOF

# 生成统计信息
echo "[5/5] 生成统计信息..."

rs_file_count=$(find "$PROJECT_DIR" -name "*.rs" | wc -l)
todo_count=$(grep -rh "^// TODO:" "$PROJECT_DIR" 2>/dev/null | wc -l)
fixme_count=$(grep -rh "^// FIXME:" "$PROJECT_DIR" 2>/dev/null | wc -l)

echo "  - Rust 文件数量: $rs_file_count"
echo "  - TODO 数量: $todo_count"
echo "  - FIXME 数量: $fixme_count"

# 生成 README 说明
cat > "$PROJECT_DIR/README_BENCHMARK.md" << EOF
# 场景三测试数据：智能体迭代调试场景

## 数据集说明

本数据集用于测试智能体多轮检索完成"找出项目中所有未处理的 TODO"的能力。

## 数据规模

- **Rust 文件数量**: $rs_file_count
- **TODO 数量**: $todo_count
- **FIXME 数量**: $fixme_count
- **模块数量**: ${#MODULES[@]}

## 使用示例

### XORE 多轮检索

\`\`\`bash
# 第1轮：找出所有 TODO
xore f "TODO" --path $PROJECT_DIR --type rs

# 第2轮：按模块分组检索
xore f "TODO" --path $PROJECT_DIR/src/auth --type rs
xore f "TODO" --path $PROJECT_DIR/src/database --type rs

# 第3轮：查找特定 TODO
xore f "TODO: Implement" --path $PROJECT_DIR --type rs
\`\`\`

### ripgrep 对比

\`\`\`bash
# 一次性找出所有 TODO
ripgrep -r "TODO" $PROJECT_DIR --type rust

# 多轮检索（模拟智能体）
ripgrep -r "TODO" $PROJECT_DIR/src/auth --type rust
ripgrep -r "TODO" $PROJECT_DIR/src/database --type rust
\`\`\`

### 对比指标

| 指标 | XORE | ripgrep |
|------|------|---------|
| 首次检索耗时 | 待测试 | 待测试 |
| Token 消耗 | 待测试 | 待测试 |
| 多轮检索支持 | ✅ 原生支持 | ❌ 需手动 |

## 生成脚本

\`\`\`bash
bash examples/benchmark-data/generate_todo_project.sh
\`\`\`
EOF

echo ""
echo "=========================================="
echo "测试数据生成完成！"
echo "=========================================="
echo ""
echo "数据位置: $PROJECT_DIR"
echo "Rust 文件: $rs_file_count"
echo "TODO: $todo_count"
echo "FIXME: $fixme_count"
echo ""
echo "运行检索测试："
echo "  xore f \"TODO\" --path $PROJECT_DIR --type rs"
echo "  ripgrep -r \"TODO\" $PROJECT_DIR --type rust"
echo ""
