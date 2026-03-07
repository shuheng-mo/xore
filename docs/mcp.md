# XORE MCP 服务器文档

本文档介绍如何配置和使用 XORE MCP 服务器，以便在 AI 编码助手（如 Roo Code）中集成 XORE 的文件搜索和数据处理能力。

---

## 目录

- [概述](#概述)
- [前置要求](#前置要求)
- [快速开始](#快速开始)
- [配置说明](#配置说明)
- [可用工具](#可用工具)
- [使用示例](#使用示例)
- [调试与故障排除](#调试与故障排除)

---

## 概述

XORE MCP 服务器是基于 [Model Context Protocol (MCP)](https://modelcontextprotocol.io/) 实现的服务器，允许 AI 助手通过标准化的接口调用 XORE 的核心功能：

- **文件搜索**：快速查找本地文件系统中的文件
- **全文索引搜索**：基于 Tantivy 的高性能全文搜索
- **数据处理**：SQL 查询、数据采样、质量检查
- **配置查询**：获取 XORE 配置信息

---

## 前置要求

- **Rust** >= 1.91.0
- **XORE** 已编译（参见 [快速开始](../getting-started.md)）
- **AI 助手**：支持 MCP 协议的客户端（如 Roo Code、Cursor、Claude Desktop 等）

---

## 快速开始

### 1. 编译 XORE MCP 服务器

```bash
# 克隆项目（如果尚未克隆）
git clone https://github.com/shuheng-mo/xore.git
cd xore

# 编译 MCP 服务器
cargo build --release -p xore-mcp
```

编译产物位于：`target/release/xore-mcp`

### 2. 配置 MCP 客户端

#### Roo Code

在项目根目录创建或编辑 `.mcp.json` 文件：

```json
{
  "mcpServers": {
    "xore": {
      "type": "stdio",
      "command": "/path/to/xore-mcp",
      "args": [],
      "env": {}
    }
  }
}
```

> **注意**：将 `/path/to/xore-mcp` 替换为实际编译路径。

#### Claude Desktop (macOS)

编辑 `~/Library/Application Support/Claude/claude_desktop_config.json`：

```json
{
  "mcpServers": {
    "xore": {
      "command": "/absolute/path/to/xore-mcp",
      "args": []
    }
  }
}
```

### 3. 重启客户端

重启 Roo Code 或 Claude Desktop，XORE MCP 服务器将自动加载。

---

## 配置说明

### 环境变量

| 变量名 | 说明 | 默认值 |
|--------|------|--------|
| `XORE_CONFIG_PATH` | 配置文件路径 | `~/.xore/config.toml` |
| `XORE_INDEX_PATH` | 索引存储路径 | `~/.xore/index` |
| `XORE_LOG_LEVEL` | 日志级别 | `info` |

### MCP 传输方式

XORE MCP 服务器使用 **stdio** 传输方式（标准输入/输出），这意味着：

- 所有通信通过 stdin/stdout 进行
- 适合本地运行的 AI 助手集成
- 日志输出到 stderr（不影响 MCP 协议通信）

---

## 可用工具

XORE MCP 服务器提供 7 个工具：

### 1. find_files

查找本地文件系统中的文件。

**参数**：

- `path` (必需)：搜索根目录
- `file_type`：文件类型过滤（如 `"csv"`, `"rs"`, `"py"`）
- `max_depth`：最大目录深度
- `include_hidden`：是否包含隐藏文件
- `limit`：返回结果数量限制（默认 200）

**示例**：

```json
{
  "name": "find_files",
  "arguments": {
    "path": ".",
    "file_type": "rs",
    "limit": 10
  }
}
```

### 2. search_index

搜索 XORE 全文索引。

**参数**：

- `query` (必需)：搜索查询
- `index_path`：索引目录路径（可选）
- `file_type`：文件类型过滤
- `limit`：返回结果数量限制（默认 50）

**示例**：

```json
{
  "name": "search_index",
  "arguments": {
    "query": "error handling",
    "file_type": "rs",
    "limit": 10
  }
}
```

### 3. get_schema

获取数据文件的 Schema 和统计信息。

**参数**：

- `path` (必需)：数据文件路径（支持 CSV, Parquet, JSON）

**示例**：

```json
{
  "name": "get_schema",
  "arguments": {
    "path": "data/sales.csv"
  }
}
```

### 4. query_data

对数据文件执行 SQL 查询。

**参数**：

- `path` (必需)：数据文件路径
- `sql` (必需)：SQL 查询（使用表名 `this`）
- `limit`：返回行数限制（默认 1000）

**示例**：

```json
{
  "name": "query_data",
  "arguments": {
    "path": "data/sales.csv",
    "sql": "SELECT * FROM this WHERE status = 'pending' LIMIT 10",
    "limit": 10
  }
}
```

### 5. sample_data

从数据文件中采样行。

**参数**：

- `path` (必需)：数据文件路径
- `rows`：采样行数（默认 20）
- `strategy`：采样策略 `"smart"` | `"head"` | `"tail"`（默认 `"smart"`）

**示例**：

```json
{
  "name": "sample_data",
  "arguments": {
    "path": "data/sales.csv",
    "rows": 5,
    "strategy": "smart"
  }
}
```

### 6. quality_check

执行数据质量检查。

**参数**：

- `path` (必需)：数据文件路径

**示例**：

```json
{
  "name": "quality_check",
  "arguments": {
    "path": "data/sales.csv"
  }
}
```

### 7. get_config

获取当前 XORE 配置。

**参数**：无

**示例**：

```json
{
  "name": "get_config",
  "arguments": {}
}
```

---

## 使用示例

### 在 Roo Code 中使用

1. 打开 Roo Code
2. 激活 XORE MCP 服务器（通常自动激活）
3. 在 AI 对话中直接请求：

```
请帮我查找当前目录下的所有 Rust 文件
```

AI 会自动调用 `find_files` 工具并返回结果。

### 手动测试 MCP 服务器

```bash
# 启动服务器并发送测试请求
MCP_BIN="/path/to/xore-mcp"

(echo '{"jsonrpc":"2.0","id":0,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'
sleep 0.3
echo '{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}'
sleep 0.3
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}') | "$MCP_BIN"
```

---

## 调试与故障排除

### 查看服务器日志

MCP 服务器的日志输出到 stderr：

```bash
# 运行服务器并查看日志
./target/release/xore-mcp 2>&1 | head -20
```

### 常见问题

#### 1. 服务器启动失败

**症状**：客户端连接超时

**排查**：

- 确认 `xore-mcp` 二进制文件存在且可执行
- 检查 `.mcp.json` 配置路径是否正确
- 尝试手动运行服务器检查错误

#### 2. 工具调用超时

**症状**：AI 调用工具时提示超时

**排查**：

- 检查搜索目录是否包含大量文件
- 尝试减小 `limit` 参数
- 考虑使用索引搜索（`search_index`）替代文件扫描（`find_files`）

#### 3. 数据文件读取失败

**症状**：`get_schema` 或 `query_data` 返回错误

**排查**：

- 确认文件路径正确
- 确认文件格式支持（CSV, Parquet, JSON）
- 检查文件权限

---

## 相关文档

- [XORE 快速开始](../getting-started.md)
- [命令参考](../commands/README.md)
- [配置参考](../reference/configuration.md)
- [Roo Code 集成](../.roo/skills/xore-agent/SKILL.md)
