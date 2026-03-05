# 命令参考

XORE 提供以下命令：

| 命令 | 别名 | 说明 |
|-----|------|------|
| [find](./find.md) | `f` | 文件搜索与扫描 |
| [process](./process.md) | `p` | 数据处理与分析（CSV/JSON/Parquet，基于 Polars）|
| [agent](./agent.md) | `agent` | **Agent-Native 接口（降低 90%+ Token 消耗）** 🚀 |
| [benchmark](./benchmark.md) | `bench` | 性能基准测试 |

## 全局选项

以下选项适用于所有命令：

| 选项 | 短选项 | 类型 | 默认值 | 说明 |
|-----|-------|------|-------|------|
| `--verbose` | `-v` | bool | false | 启用详细输出模式 |
| `--quiet` | `-q` | bool | false | 静默模式（只输出结果）|
| `--no-color` | - | bool | false | 禁用彩色输出 |
| `--help` | `-h` | - | - | 显示帮助信息 |
| `--version` | `-V` | - | - | 显示版本信息 |

## 使用示例

```bash
# 详细模式（全局）
xore --verbose find "error"

# 详细模式（子命令级别，等效）
xore find --verbose "error"

# 静默模式
xore --quiet find --type code

# 禁用颜色（适用于管道操作）
xore --no-color find "TODO" > results.txt

# 组合使用
xore -v process data.csv --quality-check
```

## 帮助系统

```bash
# 查看主帮助
xore --help

# 查看特定命令帮助
xore find --help
xore process --help
xore agent --help
xore benchmark --help

# 使用别名
xore f --help
xore p --help
xore agent --help
xore bench --help
```

## 退出码

| 退出码 | 说明 |
|-------|------|
| 0 | 成功 |
| 1 | 一般错误 |
| 2 | 命令行参数错误 |

## 相关文档

- [过滤器语法](../reference/filters.md) - 文件类型、大小、时间过滤器
- [配置文件](../reference/configuration.md) - 自定义默认行为
- [环境变量](../reference/environment.md) - 环境变量参考
