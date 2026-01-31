# 快速入门

本指南帮助你快速上手 XORE。

## 安装

### 方式一：从源码编译（推荐）

```bash
# 克隆仓库
git clone https://github.com/yourusername/xore.git
cd xore

# 编译 release 版本（启用 mimalloc 优化）
cargo build --release

# 验证安装
./target/release/xore --version
```

### 方式二：使用 Cargo

```bash
cargo install xore
```

### 方式三：下载预编译二进制

从 [Releases](https://github.com/yourusername/xore/releases) 页面下载对应平台的二进制文件。

## 基本用法

### 1. 文件搜索

```bash
# 扫描当前目录下所有文件
xore find

# 搜索包含 "error" 的文件
xore find "error"

# 只搜索 Rust 源文件
xore find --type code

# 搜索大于 1MB 的 CSV 文件
xore find --type csv --size ">1MB"

# 搜索最近 7 天修改的文件
xore find --mtime "-7d"
```

### 2. 数据处理

```bash
# 预览 CSV 文件
xore process data.csv

# 检查数据质量
xore process data.csv --quality-check

# 执行 SQL 查询（开发中）
xore process data.csv "SELECT * FROM self WHERE age > 30"
```

### 3. 性能测试

```bash
# 运行所有基准测试
xore benchmark

# 只测试文件扫描性能
xore benchmark --suite scan

# 测试内存分配性能
xore benchmark --suite alloc

# 输出 JSON 格式结果
xore benchmark --output json
```

## 常用场景

### 场景 1：查找大文件

```bash
# 查找大于 100MB 的文件
xore find --size ">100MB"

# 查找 10MB 到 100MB 之间的日志文件
xore find --type log --size "10MB-100MB"
```

### 场景 2：清理旧文件

```bash
# 查找超过 30 天未修改的文件
xore find --mtime "+30d"

# 查找超过 1 年的临时文件
xore find --mtime "+365d" --type text
```

### 场景 3：代码审查

```bash
# 查找所有 TODO 注释
xore find "TODO" --type code

# 查找最近修改的代码文件
xore find --type code --mtime "-7d"
```

### 场景 4：数据质量检查

```bash
# 检查 CSV 数据质量
xore process sales.csv --quality-check

# 预览 JSON 数据结构
xore process config.json
```

## 全局选项

所有命令都支持以下全局选项：

| 选项 | 说明 |
|-----|------|
| `-v, --verbose` | 详细输出模式 |
| `-q, --quiet` | 静默模式 |
| `--no-color` | 禁用彩色输出 |
| `--help` | 显示帮助信息 |
| `--version` | 显示版本信息 |

## 命令别名

为了提高效率，XORE 提供了命令别名：

| 完整命令 | 别名 |
|---------|------|
| `xore find` | `xore f` |
| `xore process` | `xore p` |
| `xore benchmark` | `xore bench` |

## 下一步

- 阅读 [命令参考](./commands/README.md) 了解所有命令的详细参数
- 查看 [过滤器语法](./reference/filters.md) 学习高级过滤技巧
- 了解 [配置文件](./reference/configuration.md) 自定义 XORE 行为
