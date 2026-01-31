# 配置文件参考

XORE 使用 TOML 格式的配置文件来自定义默认行为。

## 配置文件位置

| 平台 | 默认路径 |
|------|---------|
| Linux | `~/.config/xore/config.toml` |
| macOS | `~/.config/xore/config.toml` |
| Windows | `%APPDATA%\xore\config.toml` |

也可以通过环境变量 `XORE_CONFIG_PATH` 指定自定义路径。

## 配置文件结构

```toml
# XORE 配置文件示例

[search]
# 索引存储路径
index_path = "~/.xore/index"
# 默认搜索线程数（0 = 自动检测）
threads = 0
# 自动重建索引间隔（秒）
auto_rebuild_interval = 3600

[process]
# 是否使用懒加载
lazy = true
# 数据块大小
chunk_size = 65536
# 缓存大小（MB）
cache_size_mb = 256

[ai]
# ONNX 模型路径
model_path = "~/.xore/models"
# 是否启用语义搜索
semantic_enabled = false

[limits]
# 最大内存使用（MB）
max_memory_mb = 1024
# 最大文件大小（MB）
max_file_size_mb = 100
# 查询超时（秒）
query_timeout_secs = 30

[ui]
# 主题（light/dark/auto）
theme = "auto"
# 是否显示进度条
progress_bar = true
# 是否使用彩色输出
color = true

[exclude]
# 全局排除模式
patterns = [
    "**/node_modules/**",
    "**/.git/**",
    "**/target/**",
    "**/__pycache__/**",
]
```

## 配置项详解

### [search] 搜索配置

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `index_path` | String | `~/.xore/index` | 索引文件存储路径 |
| `threads` | usize | `0` | 搜索线程数，0 表示自动检测 CPU 核心数 |
| `auto_rebuild_interval` | u64 | `3600` | 自动重建索引的间隔（秒）|

### [process] 数据处理配置

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `lazy` | bool | `true` | 是否启用懒加载模式 |
| `chunk_size` | usize | `65536` | 数据处理块大小（字节）|
| `cache_size_mb` | usize | `256` | 内存缓存大小（MB）|

### [ai] AI 配置

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `model_path` | String | `~/.xore/models` | ONNX 模型存储路径 |
| `semantic_enabled` | bool | `false` | 是否启用语义搜索 |

### [limits] 资源限制

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `max_memory_mb` | usize | `1024` | 最大内存使用（MB）|
| `max_file_size_mb` | usize | `100` | 单个文件最大大小（MB）|
| `query_timeout_secs` | u64 | `30` | 查询超时时间（秒）|

### [ui] 界面配置

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `theme` | String | `"auto"` | 主题：`light`, `dark`, `auto` |
| `progress_bar` | bool | `true` | 是否显示进度条 |
| `color` | bool | `true` | 是否使用彩色输出 |

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
]
```

## 命令行覆盖

命令行参数优先级高于配置文件：

```bash
# 配置文件设置 threads = 4
# 命令行覆盖为 8
xore find --threads 8
```

## 配置文件示例

### 最小配置

```toml
[ui]
color = true

[search]
threads = 4
```

### 开发环境配置

```toml
[search]
threads = 8

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
[search]
threads = 0  # 自动检测

[limits]
max_memory_mb = 4096
max_file_size_mb = 500
query_timeout_secs = 60

[ui]
progress_bar = false
color = false
```

## 另请参阅

- [环境变量参考](./environment.md)
- [命令参考](../commands/README.md)
