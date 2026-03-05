# 场景三测试数据：智能体迭代调试场景

## 数据集说明

本数据集用于测试智能体多轮检索完成"找出项目中所有未处理的 TODO"的能力。

## 数据规模

- **Rust 文件数量**:       57
- **TODO 数量**:      155
- **FIXME 数量**:       10
- **模块数量**: 8

## 使用示例

### XORE 多轮检索

```bash
# 第1轮：找出所有 TODO
xore f "TODO" --path examples/benchmark-data/todo_project --type rs

# 第2轮：按模块分组检索
xore f "TODO" --path examples/benchmark-data/todo_project/src/auth --type rs
xore f "TODO" --path examples/benchmark-data/todo_project/src/database --type rs

# 第3轮：查找特定 TODO
xore f "TODO: Implement" --path examples/benchmark-data/todo_project --type rs
```

### ripgrep 对比

```bash
# 一次性找出所有 TODO
ripgrep -r "TODO" examples/benchmark-data/todo_project --type rust

# 多轮检索（模拟智能体）
ripgrep -r "TODO" examples/benchmark-data/todo_project/src/auth --type rust
ripgrep -r "TODO" examples/benchmark-data/todo_project/src/database --type rust
```

### 对比指标

| 指标 | XORE | ripgrep |
|------|------|---------|
| 首次检索耗时 | 待测试 | 待测试 |
| Token 消耗 | 待测试 | 待测试 |
| 多轮检索支持 | ✅ 原生支持 | ❌ 需手动 |

## 生成脚本

```bash
bash examples/benchmark-data/generate_todo_project.sh
```
