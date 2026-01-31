# 过滤器语法参考

本文档详细说明 XORE 支持的各种过滤器语法。

## 文件类型过滤器 (`--type`)

### 预定义类型

| 类型 | 说明 | 匹配的扩展名 |
|------|------|-------------|
| `csv` | CSV 数据文件 | `.csv` |
| `json` | JSON 文件 | `.json` |
| `log` | 日志文件 | `.log` |
| `code` | 源代码文件 | `.rs`, `.py`, `.js`, `.ts`, `.go`, `.java`, `.c`, `.cpp`, `.h`, `.hpp`, `.rb`, `.php`, `.swift`, `.kt`, `.scala`, `.sh`, `.bash` |
| `text` | 文本文件 | `.txt`, `.md`, `.rst`, `.adoc` |
| `parquet` | Parquet 文件 | `.parquet` |

### 自定义扩展名

使用逗号分隔的扩展名列表：

```bash
# 搜索 XML 和 YAML 文件
xore find --type "xml,yaml,yml"

# 搜索配置文件
xore find --type "toml,ini,conf,cfg"

# 搜索图片文件
xore find --type "jpg,jpeg,png,gif,webp"
```

### 示例

```bash
# 预定义类型
xore find --type csv
xore find --type code
xore find --type log

# 自定义类型
xore find --type "xml,yaml"
xore find -t "toml,json"
```

---

## 文件大小过滤器 (`--size`)

### 推荐语法（无需引号）

| 格式 | 说明 | 示例 |
|------|------|------|
| `gt:SIZE` | 大于指定大小 | `gt:1MB` |
| `lt:SIZE` | 小于指定大小 | `lt:500KB` |
| `eq:SIZE` | 等于指定大小 | `eq:1GB` |
| `MIN-MAX` | 在指定范围内 | `1MB-10MB` |

### 兼容语法（需要引号）

| 格式 | 说明 | 示例 |
|------|------|------|
| `>SIZE` | 大于指定大小 | `">1MB"` |
| `<SIZE` | 小于指定大小 | `"<500KB"` |
| `=SIZE` | 等于指定大小 | `=1GB` |

> **推荐：** 使用 `gt:`/`lt:`/`eq:` 语法可以避免 shell 引号问题。

### 支持的单位

| 单位 | 字节数 | 示例 |
|------|--------|------|
| `B` | 1 | `1024B` |
| `KB` | 1,024 | `500KB` |
| `MB` | 1,048,576 | `10MB` |
| `GB` | 1,073,741,824 | `1GB` |

### 小数支持

支持小数值：

```bash
xore find --size gt:1.5MB
xore find --size 0.5GB-1.5GB
```

### 示例

```bash
# 大于 1MB（推荐语法）
xore find --size gt:1MB

# 小于 500KB
xore find --size lt:500KB

# 等于 1GB
xore find --size eq:1GB

# 1MB 到 10MB 之间
xore find --size 1MB-10MB

# 大于 100MB 的日志文件
xore find --type log --size gt:100MB

# 兼容语法（需要引号）
xore find --size ">1MB"
xore find --size "<500KB"
```

---

## 修改时间过滤器 (`--mtime`)

### 语法格式

| 格式 | 说明 | 示例 | 需要引号 |
|------|------|------|---------|
| `-Nd` | 最近 N 天内修改 | `-7d` | 否 |
| `+Nd` | 超过 N 天未修改 | `+30d` | 否 |
| `YYYY-MM-DD` | 指定日期之后修改 | `2024-01-01` | 否 |

> **提示：** 所有时间过滤器格式均无需引号。

### 时间单位

| 单位 | 说明 | 示例 |
|------|------|------|
| `d` | 天 | `-7d`, `+30d` |
| `h` | 小时 | `-24h`, `+72h` |

### 示例

```bash
# 最近 7 天内修改
xore find --mtime "-7d"

# 最近 24 小时内修改
xore find --mtime "-24h"

# 超过 30 天未修改
xore find --mtime "+30d"

# 超过 1 年未修改
xore find --mtime "+365d"

# 2024 年之后修改
xore find --mtime "2024-01-01"

# 组合使用
xore find --type log --mtime -7d --size gt:1MB
```

---

## 组合过滤器

过滤器可以自由组合使用，条件之间是 AND 关系：

```bash
# 最近 7 天修改的大型 CSV 文件（无需引号）
xore find --type csv --size gt:10MB --mtime -7d

# 超过 30 天的小型日志文件
xore find --type log --size lt:1MB --mtime +30d

# 最近修改的代码文件（限制深度）
xore find --type code --mtime "-1d" --max-depth 3
```

---

## 特殊选项

### 隐藏文件 (`--hidden`)

默认情况下，XORE 不扫描以 `.` 开头的隐藏文件和目录。

```bash
# 包含隐藏文件
xore find --hidden

# 搜索隐藏的配置文件
xore find --hidden --type "rc,conf"
```

### .gitignore 规则 (`--no-ignore`)

默认情况下，XORE 遵守 `.gitignore` 规则。

```bash
# 不遵守 .gitignore（搜索所有文件）
xore find --no-ignore

# 搜索被忽略的 node_modules
xore find --no-ignore --path node_modules
```

### 符号链接 (`--follow-links`)

默认情况下，XORE 不跟随符号链接。

```bash
# 跟随符号链接
xore find --follow-links
```

---

## 性能提示

1. **使用类型过滤器**：预先过滤文件类型可以显著减少扫描量
2. **限制深度**：使用 `--max-depth` 限制目录深度
3. **利用 .gitignore**：默认会跳过 .gitignore 中的文件
4. **调整线程数**：使用 `--threads` 根据系统调整并行度

```bash
# 高效搜索示例
xore find "error" --type log --max-depth 5 --threads 8
```
