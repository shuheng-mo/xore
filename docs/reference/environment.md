# 环境变量参考

XORE 支持通过环境变量配置运行时行为。

## 环境变量列表

| 变量 | 说明 | 默认值 | 示例 |
|------|------|--------|------|
| `XORE_CONFIG_PATH` | 配置文件路径 | `~/.config/xore/config.toml` | `/etc/xore/config.toml` |
| `XORE_INDEX_PATH` | 索引存储路径 | `~/.xore/index` | `/var/xore/index` |
| `XORE_HISTORY_PATH` | 搜索历史存储路径 | `~/.xore/history` | `/var/xore/history` |
| `XORE_LOG_LEVEL` | 日志级别 | `info` | `debug`, `trace` |
| `XORE_NUM_THREADS` | 工作线程数 | CPU 核心数 | `8` |
| `NO_COLOR` | 禁用彩色输出 | 未设置 | `1` |
| `RUST_LOG` | Rust 日志过滤器 | 未设置 | `xore=debug` |

## 详细说明

### XORE_CONFIG_PATH

指定配置文件的自定义路径。

```bash
# 使用自定义配置文件
export XORE_CONFIG_PATH=/etc/xore/config.toml
xore find

# 临时使用不同配置
XORE_CONFIG_PATH=./dev.toml xore find
```

### XORE_INDEX_PATH

指定索引文件的存储位置。

```bash
# 将索引存储到指定目录
export XORE_INDEX_PATH=/data/xore/index
xore find
```

### XORE_HISTORY_PATH

指定搜索历史记录的存储位置。

```bash
# 将历史记录存储到指定目录
export XORE_HISTORY_PATH=/data/xore/history
xore find "error"
```

> **注意：** 历史记录以 JSON 格式存储在 `history.json` 文件中。

### XORE_LOG_LEVEL

设置日志级别。

| 级别 | 说明 |
|------|------|
| `error` | 只显示错误 |
| `warn` | 显示警告和错误 |
| `info` | 显示信息、警告和错误（默认）|
| `debug` | 显示调试信息 |
| `trace` | 显示所有信息 |

```bash
# 启用调试日志
export XORE_LOG_LEVEL=debug
xore find

# 静默模式
export XORE_LOG_LEVEL=error
xore find
```

### XORE_NUM_THREADS

设置并行工作线程数。

```bash
# 使用 8 个线程
export XORE_NUM_THREADS=8
xore find

# 单线程模式（调试用）
export XORE_NUM_THREADS=1
xore find
```

### NO_COLOR

遵循 [no-color.org](https://no-color.org/) 标准，禁用彩色输出。

```bash
# 禁用颜色（适用于管道操作）
export NO_COLOR=1
xore find > results.txt

# 或使用命令行参数
xore --no-color find
```

### RUST_LOG

Rust 生态系统标准的日志过滤器，用于细粒度日志控制。

```bash
# 只显示 xore 模块的 debug 日志
export RUST_LOG=xore=debug
xore find

# 显示所有模块的 trace 日志
export RUST_LOG=trace
xore find

# 组合过滤
export RUST_LOG=xore_search=debug,xore_core=info
xore find
```

## 优先级

配置的优先级从高到低：

1. **命令行参数** - 最高优先级
2. **环境变量**
3. **配置文件**
4. **默认值** - 最低优先级

```bash
# 配置文件设置 threads = 4
# 环境变量设置 XORE_NUM_THREADS=8
# 命令行参数 --threads 16

# 最终使用 16 个线程
xore find --threads 16
```

## 使用示例

### 开发环境

```bash
# .bashrc 或 .zshrc
export XORE_LOG_LEVEL=debug
export XORE_NUM_THREADS=4
export XORE_CONFIG_PATH=~/.config/xore/dev.toml
```

### CI/CD 环境

```bash
# 禁用交互式输出
export NO_COLOR=1
export XORE_LOG_LEVEL=error

# 运行测试
xore find --type code
```

### Docker 容器

```dockerfile
ENV XORE_CONFIG_PATH=/app/config/xore.toml
ENV XORE_INDEX_PATH=/data/index
ENV XORE_NUM_THREADS=2
ENV NO_COLOR=1
```

### Shell 脚本

```bash
#!/bin/bash

# 设置环境
export NO_COLOR=1
export XORE_LOG_LEVEL=error

# 执行搜索并保存结果
xore find --type log --size ">10MB" > large_logs.txt
```

## 调试技巧

### 查看当前配置

```bash
# 启用详细输出查看配置信息（全局）
xore --verbose find --help

# 启用详细输出查看配置信息（子命令级别，等效）
xore find --verbose --help
```

### 排查问题

```bash
# 启用最详细的日志
export RUST_LOG=trace
xore find 2>&1 | head -100
```

### 性能分析

```bash
# 使用 benchmark 命令
export XORE_NUM_THREADS=8
xore benchmark --suite scan -n 10
```

## 另请参阅

- [配置文件参考](./configuration.md)
- [命令参考](../commands/README.md)
