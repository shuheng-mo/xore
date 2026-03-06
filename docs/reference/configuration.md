# 配置文件参考

XORE 使用 TOML 格式的配置文件来自定义默认行为。

## 配置文件位置

| 平台 | 默认路径 |
|------|---------|
| Linux | `~/.xore/config.toml` |
| macOS | `~/.xore/config.toml` |
| Windows | `%USERPROFILE%\.xore\config.toml` |

> **注意：** 从 v1.0.0 起，配置文件位置从 `~/.config/xore/config.toml` 迁移到 `~/.xore/config.toml`。

也可以通过环境变量 `XORE_CONFIG_PATH` 指定自定义路径。

## 配置文件结构

```toml
# XORE 配置文件 - 极简设计

# 运行时环境配置
[env]
# 日志级别: error, warn, info, debug, trace
log_level = "info"
# 工作线程数（0 = 自动检测 CPU 核心数）
num_threads = 0

# 存储路径配置
[paths]
# 索引存储路径
index = "~/.xore/index"
# 历史记录存储路径
history = "~/.xore/history"
# 日志存储路径
logs = "~/.xore/logs"
# AI 模型存储路径
models = "~/.xore/models"

# 搜索配置
[search]
# 是否使用项目级索引（优先于全局索引）
use_project_index = true
# 项目级索引路径（相对于项目根目录）
project_index_path = ".xore/index"
# 单文件最大大小（MB），超过不索引
max_file_size_mb = 100
# 索引 Writer 缓冲区大小（MB），最小 15MB
writer_buffer_mb = 50

# 排除模式
[exclude]
patterns = [
    "**/node_modules/**",
    "**/.git/**",
    "**/target/**",
    "**/__pycache__/**",
    "**/.DS_Store/**",
    "**/Thumbs.db/**",
]

# 界面配置
[ui]
# 主题（light/dark/auto）
theme = "auto"
# 是否显示进度条
progress_bar = true
# 是否使用彩色输出
color = true
```

## 配置项详解

### [env] 运行时环境

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `log_level` | String | `"info"` | 日志级别 |
| `num_threads` | usize | `0` | 工作线程数，0 表示自动检测 |

### [paths] 存储路径

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `index` | PathBuf | `~/.xore/index` | 索引存储路径 |
| `history` | PathBuf | `~/.xore/history` | 历史记录存储路径 |
| `logs` | PathBuf | `~/.xore/logs` | 日志存储路径 |
| `models` | PathBuf | `~/.xore/models` | AI 模型存储路径 |

### [search] 搜索配置

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `use_project_index` | bool | `true` | 是否使用项目级索引 |
| `project_index_path` | String | `.xore/index` | 项目级索引路径 |
| `max_file_size_mb` | usize | `100` | 单文件最大大小（MB）|
| `writer_buffer_mb` | usize | `50` | 索引 Writer 缓冲区大小（MB）|

### [exclude] 排除配置

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `patterns` | Array | 见下文 | 全局排除的 glob 模式 |

默认排除模式：

```toml
patterns = [
    "**/node_modules/**",
    "**/.git/**",
    "**/target/**",
    "**/__pycache__/**",
    "**/.DS_Store/**",
    "**/Thumbs.db/**",
]
```

### [ui] 界面配置

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `theme` | String | `"auto"` | 主题：`light`, `dark`, `auto` |
| `progress_bar` | bool | `true` | 是否显示进度条 |
| `color` | bool | `true` | 是否使用彩色输出 |

## 命令行覆盖

命令行参数优先级高于配置文件：

```bash
# 配置文件设置 num_threads = 4
# 命令行覆盖为 8
xore find --threads 8
```

## 配置文件示例

### 最小配置

```toml
[ui]
color = true

[env]
log_level = "info"
```

### 开发环境配置

```toml
[env]
log_level = "debug"
num_threads = 8

[ui]
progress_bar = true
color = true

[exclude]
patterns = [
    "**/node_modules/**",
    "**/target/**",
    "**/.git/**",
    "**/dist/**",
    "**/build/**",
]
```

### 生产环境配置

```toml
[env]
log_level = "warn"
num_threads = 0  # 自动检测

[search]
max_file_size_mb = 500

[ui]
progress_bar = false
color = false
```

## 目录结构

安装 XORE 后，会在用户主目录下创建以下目录结构：

```
~/.xore/
├── config.toml    # 全局配置文件
├── index/         # 搜索索引存储
│   └── default/   # 默认索引
├── history/       # 搜索历史记录
│   └── history.json
├── logs/          # 运行日志
│   └── xore.log
├── models/        # AI 模型存储
│   └── minilm-l6-v2.onnx
└── cache/         # 缓存文件
```

## 另请参阅

- [环境变量参考](./environment.md)
- [命令参考](../commands/README.md)
