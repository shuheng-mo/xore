# find 命令

文件搜索与扫描命令。

**别名:** `f`

## 语法

```bash
xore find [OPTIONS] [QUERY]
xore f [OPTIONS] [QUERY]
```

## 描述

`find` 命令用于在指定目录下搜索文件。支持：
- 文件名和内容搜索
- 多种过滤条件（类型、大小、修改时间）
- 并行扫描
- .gitignore 规则遵守

## 参数

### 位置参数

| 参数 | 类型 | 必填 | 说明 |
|-----|------|-----|------|
| `QUERY` | String | 否 | 搜索字符串，省略则只扫描文件 |

### 选项参数

| 选项 | 短选项 | 类型 | 默认值 | 说明 |
|-----|-------|------|-------|------|
| `--path` | - | String | `.` | 搜索目录路径 |
| `--type` | `-t` | String | - | 文件类型过滤器 |
| `--size` | `-s` | String | - | 文件大小过滤器 |
| `--mtime` | `-m` | String | - | 修改时间过滤器 |
| `--max-depth` | `-d` | usize | 无限 | 最大目录遍历深度 |
| `--hidden` | - | bool | false | 包含隐藏文件 |
| `--no-ignore` | - | bool | false | 不遵守 .gitignore 规则 |
| `--follow-links` | `-L` | bool | false | 跟随符号链接 |
| `--threads` | `-j` | usize | 自动 | 并行线程数 |
| `--semantic` | - | bool | false | 启用语义搜索（开发中）|

## 过滤器语法

### 文件类型 (`--type`)

| 值 | 说明 | 扩展名 |
|---|------|-------|
| `csv` | CSV 数据文件 | .csv |
| `json` | JSON 文件 | .json |
| `log` | 日志文件 | .log |
| `code` | 源代码文件 | .rs, .py, .js, .go, .java 等 |
| `text` | 文本文件 | .txt, .md, .rst 等 |
| `parquet` | Parquet 文件 | .parquet |
| 自定义 | 逗号分隔的扩展名 | 如 `xml,yaml,toml` |

### 文件大小 (`--size`)

| 格式 | 示例 | 说明 | 需要引号 |
|-----|------|------|---------|
| `>SIZE` | `">1MB"` | 大于指定大小 | 是（`>` 是 shell 重定向符）|
| `<SIZE` | `"<500KB"` | 小于指定大小 | 是（`<` 是 shell 重定向符）|
| `=SIZE` | `=1GB` | 等于指定大小 | 否 |
| `MIN-MAX` | `1MB-10MB` | 在指定范围内 | 否 |

支持单位：`B`, `KB`, `MB`, `GB`

> **提示：** 使用 `>` 或 `<` 时必须加引号，因为这些是 shell 重定向符号。范围格式 `1MB-10MB` 无需引号。

### 修改时间 (`--mtime`)

| 格式 | 示例 | 说明 | 需要引号 |
|-----|------|------|---------|
| `-Nd` | `-7d` | 最近 N 天内修改 | 否 |
| `+Nd` | `+30d` | 超过 N 天未修改 | 否 |
| `YYYY-MM-DD` | `2024-01-01` | 指定日期之后 | 否 |

> **提示：** 所有时间格式均无需引号。

## 使用示例

### 基本搜索

```bash
# 扫描当前目录所有文件
xore find

# 搜索包含 "error" 的文件
xore find "error"

# 在指定目录搜索
xore find "config" --path /etc
```

### 类型过滤

```bash
# 只搜索 Rust 代码
xore find --type code

# 搜索 CSV 文件
xore find --type csv

# 搜索自定义扩展名
xore find --type "xml,yaml,toml"
```

### 大小过滤

```bash
# 大于 1MB 的文件
xore find --size ">1MB"

# 小于 500KB 的文件
xore find --size "<500KB"

# 1MB 到 10MB 之间
xore find --size "1MB-10MB"
```

### 时间过滤

```bash
# 最近 7 天修改的文件
xore find --mtime "-7d"

# 超过 30 天未修改
xore find --mtime "+30d"

# 2024 年之后修改
xore find --mtime "2024-01-01"
```

### 组合过滤

```bash
# 最近修改的大型日志文件
xore find --type log --size ">10MB" --mtime "-7d"

# 深度限制搜索
xore find "TODO" --type code --max-depth 3

# 包含隐藏文件
xore find --hidden --type text
```

### 高级选项

```bash
# 不遵守 .gitignore
xore find --no-ignore

# 跟随符号链接
xore find --follow-links

# 指定线程数
xore find --threads 8

# 详细输出
xore --verbose find "error"
```

## 输出格式

```
🔍 扫描文件中...

   2.39 KB  ./src/main.rs
   4.40 KB  ./src/lib.rs
  10.23 KB  ./src/scanner.rs

✓ 找到 3 个文件 (共扫描 45 个文件, 12 个目录, 耗时 5 ms)
  总大小: 17.02 KB
  已跳过: 42 个文件 (不匹配过滤条件)
```

## 相关命令

- [process](./process.md) - 处理找到的数据文件
- [benchmark](./benchmark.md) - 测试扫描性能

## 另请参阅

- [过滤器语法参考](../reference/filters.md)
