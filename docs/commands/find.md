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
- **全文索引搜索**（基于 Tantivy，支持中英文）
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
| `--index` | `-i` | bool | false | 启用全文索引搜索模式 |
| `--rebuild` | - | bool | false | 强制重建索引 |
| `--index-dir` | - | String | `.xore/index` | 指定索引目录路径 |
| `--watch` | `-w` | bool | false | 启用文件监控与增量索引 |
| `--semantic` | - | bool | false | 启用语义搜索（基于 ONNX）|
| `--history` | - | bool | false | 显示搜索历史记录 |
| `--recommend` | - | bool | false | 显示智能推荐 |
| `--clear-history` | - | bool | false | 清除搜索历史记录 |

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

**推荐语法（无需引号）：**

| 格式 | 示例 | 说明 |
|-----|------|------|
| `gt:SIZE` | `gt:1MB` | 大于指定大小 |
| `lt:SIZE` | `lt:500KB` | 小于指定大小 |
| `eq:SIZE` | `eq:1GB` | 等于指定大小 |
| `MIN-MAX` | `1MB-10MB` | 在指定范围内 |

**兼容语法（需要引号）：**

| 格式 | 示例 | 说明 |
|-----|------|------|
| `>SIZE` | `">1MB"` | 大于（需引号，`>` 是 shell 重定向符）|
| `<SIZE` | `"<500KB"` | 小于（需引号，`<` 是 shell 重定向符）|

支持单位：`B`, `KB`, `MB`, `GB`

> **提示：** 推荐使用 `gt:`/`lt:`/`eq:` 语法，无需担心 shell 引号问题。

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
# 大于 1MB 的文件（推荐语法）
xore find --size gt:1MB

# 小于 500KB 的文件
xore find --size lt:500KB

# 等于 10MB
xore find --size eq:10MB

# 1MB 到 10MB 之间
xore find --size 1MB-10MB

# 兼容语法（需要引号）
xore find --size ">1MB"
xore find --size "<500KB"
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
# 最近修改的大型日志文件（无需引号）
xore find --type log --size gt:10MB --mtime -7d

# 深度限制搜索
xore find "TODO" --type code --max-depth 3

# 包含隐藏文件
xore find --hidden --type text

# 超过 30 天未修改的小型文件
xore find --size lt:1MB --mtime +30d
```

### 搜索历史与智能推荐

```bash
# 执行搜索时会自动记录搜索历史
xore find "error" --path ./src

# 显示搜索历史
xore find --history

# 显示智能推荐（基于搜索频率）
xore find --recommend

# 清除搜索历史
xore find --clear-history
```

> **注意：** 搜索历史存储在 `~/.xore/history/history.json`

### 全文索引搜索

使用 `--index` 启用基于 Tantivy 的全文索引搜索，支持中英文混合搜索。

#### 标准搜索

```bash
# 启用全文索引搜索（首次使用会自动构建索引）
xore find "error" --index

# 中文关键词搜索
xore find "错误" --index

# 中英混合搜索
xore find "error 错误" --index

# 强制重建索引
xore find "test" --index --rebuild

# 指定索引目录
xore find "config" --index --index-dir /path/to/index

# 配合类型过滤
xore find "TODO" --index --type rs

# 搜索日志文件中的错误
xore find "exception" --index --type log
```

#### 前缀搜索

使用 `*` 后缀进行前缀匹配，快速查找以指定前缀开头的词：

```bash
# 搜索以 "config" 开头的词
xore find "config*" --index

# 搜索以 "err" 开头的词
xore find "err*" --index

# 在代码文件中搜索函数前缀
xore find "fn_*" --index --type rs

# 搜索配置相关内容
xore find "conf*" --index --type toml
```

**匹配示例：**

- `config*` → config, configuration, configure, configurable
- `err*` → error, errors, errno, erroneous
- `data*` → data, database, dataset, dataframe

**注意事项：**

- 前缀长度至少 2 个字符
- 前缀搜索在分词后的 token 上进行
- 对于复合词可能不符合预期，建议使用标准搜索

**性能：** p99 延迟 < 1ms

#### 模糊搜索

使用 `~` 前缀进行模糊匹配，容忍拼写错误（Levenshtein 距离 ≤ 2）：

```bash
# 搜索 "databse"（拼写错误）
xore find "~databse" --index  # 可以匹配 "database"

# 搜索 "eror"（拼写错误）
xore find "~eror" --index  # 可以匹配 "error"

# 搜索 "confg"（拼写错误）
xore find "~confg" --index  # 可以匹配 "config"

# 在日志中搜索可能拼错的关键词
xore find "~warining" --index --type log  # 匹配 "warning"
```

**容错范围：**

- Levenshtein 距离 ≤ 2
- 可以容忍：插入、删除、替换字符

**适用场景：**

- 不确定拼写
- 快速输入
- 模糊记忆
- 处理用户输入错误

**性能：** p99 延迟 < 1ms

#### 智能搜索

XORE 会自动检测查询类型，无需额外参数：

```bash
# 自动识别为前缀搜索
xore find "config*" --index

# 自动识别为模糊搜索
xore find "~databse" --index

# 自动识别为标准搜索
xore find "error" --index
```

**组合使用：**

```bash
# 前缀搜索 + 文件类型过滤
xore find "err*" --index --type log

# 模糊搜索 + 路径限制
xore find "~databse" --index --path ./src

# 标准搜索 + 增量监控
xore find "TODO" --index --watch
```

**索引搜索特性：**

- 支持中英文混合分词（基于 jieba-rs）
  - 英文：按空格和标点分割，转小写
  - 中文：使用 jieba `cut_for_search` 模式
  - 自动检测 CJK 字符，按语言块分别处理
- BM25 相关性排序
- 结果高亮显示（匹配片段黄色粗体）
- 自动跳过二进制文件（前 8000 字节检测）
- 增量更新（重复路径自动覆盖）
- 支持 `--rebuild` 强制重建索引

**索引存储位置：**

- 默认：`.xore/index`（项目级索引，可通过配置调整）
- 可通过 `--index-dir` 自定义
- 索引会持久化，再次搜索无需重建
- 最大支持索引大小可通过配置限制

**性能数据（基于 9.5GB 测试数据）：**

| 操作 | 性能指标 | 状态 |
|------|---------|------|
| 索引构建 | 92,678 MB/s | ✅ 远超目标 |
| 标准搜索 (p99) | 0.2 ms | ✅ 远超目标 |
| 前缀搜索 (p99) | 0.0 ms | ✅ 远超目标 |
| 模糊搜索 (p99) | 0.3 ms | ✅ 远超目标 |
| 增量索引延迟 | ~45 ms | ✅ 达标 |

详细性能报告：[Performance Report](../../plans/performance-report.md)

### 增量索引与文件监控 (`--watch`)

使用 `--watch` 启用实时文件监控模式，自动检测文件变更并更新全文索引：

```bash
# 启动增量监控模式
xore find "error" --index --watch

# 指定监控路径
xore find "TODO" --index --watch --path ./src

# 配合文件类型过滤
xore find "test" --index --watch -t rs

# 按 Ctrl+C 停止监控
```

**工作原理：**

1. **初始索引**：启动时检查并构建初始索引
2. **文件监控**：监听目录的文件创建、修改、删除事件
3. **增量更新**：
   - 文件创建 → 扫描并添加到索引
   - 文件修改 → 删除旧文档，重新扫描并添加
   - 文件删除 → 从索引中删除
4. **防抖处理**：500ms 内同一文件的多次变更合并为一次
5. **批量提交**：累积 50 个变更或每 30 秒自动提交

**监控配置：**

| 配置项 | 值 | 说明 |
|--------|-----|------|
| 防抖时间 | 500ms | 多次变更合并等待时间 |
| 批量提交阈值 | 50 个 | 自动提交前的变更数量 |
| 自动提交间隔 | 30 秒 | 无变更时的提交周期 |
| WAL 记录数 | 1000 条 | 内存中保留的操作日志 |

**排除规则：**

- 遵守 `.gitignore` 和 `.opencodeignore` 规则
- 自动排除临时文件：`*.tmp`, `*.swp`, `*.swo` 等
- 自动排除系统目录：`.git`, `node_modules`, `target` 等

**注意事项：**

- 需要在 Git 仓库中运行，否则 `.gitignore` 规则不生效
- Windows 上可能需要以管理员权限运行
- 大量文件变更时，防抖机制会延迟索引更新

### 语义搜索 (`--semantic`)

使用 `--semantic` 启用基于 ONNX 的语义搜索，通过理解文本含义而非关键词匹配来查找相关文件：

```bash
# 语义搜索
xore find "数据库连接失败的相关代码" --semantic

# 限制搜索路径
xore find "错误处理逻辑" --semantic --path ./src

# 配合文件类型过滤
xore find "配置文件示例" --semantic --type toml
```

**工作原理：**

1. **模型加载**：加载预训练的 MiniLM-L6-v2 模型（384维向量）
2. **文件索引**：读取文件内容并生成嵌入向量
3. **语义匹配**：计算查询与文档的余弦相似度
4. **结果排序**：按相似度从高到低返回结果

**模型配置：**

| 配置项 | 默认值 | 环境变量 |
|--------|--------|----------|
| 模型路径 | `assets/models/onnx/model.onnx` | `XORE_MODEL_PATH` |
| Tokenizer 路径 | `assets/models/tokenizer.json` | `XORE_TOKENIZER_PATH` |

**使用前准备：**

1. 下载模型文件（参考 [语义搜索指南](../semantic-search-guide.md)）
2. 确保模型文件在正确路径
3. 或通过环境变量指定自定义路径

```bash
# 使用自定义模型路径
export XORE_MODEL_PATH=/path/to/model.onnx
export XORE_TOKENIZER_PATH=/path/to/tokenizer.json
xore find "查询内容" --semantic
```

**性能特性：**

| 指标 | 目标值 | 说明 |
|------|--------|------|
| 模型加载 | <2s | 首次加载 |
| 单文件嵌入 | <100ms | CPU 推理 |
| 向量搜索 | <100ms | 10K 文档 |
| 最大索引文件数 | 1000 | 防止内存溢出 |
| 最大文件大小 | 1MB | 跳过过大文件 |

**适用场景：**

- 概念性搜索（"如何处理错误"）
- 跨语言搜索（中英文混合）
- 模糊意图搜索（不确定关键词）
- 代码功能查找（"实现了什么功能"）

**限制：**

- 需要预先下载模型文件（~80MB）
- 首次搜索需要索引所有文件（较慢）
- 仅索引文本文件，跳过二进制文件
- 内存占用较高（取决于文件数量）

**示例输出：**

```bash
$ xore find "错误处理" --semantic --path ./src

正在加载语义搜索模型...
✓ 模型加载成功
正在索引文件内容...
████████████████████████████████████████ 156/156
✓ 已索引 156 个文件 (跳过 12 个)
正在搜索: "错误处理"
  src/error.rs (相似度: 0.8234)
  src/lib.rs (相似度: 0.7156)
  src/commands/process.rs (相似度: 0.6892)
  ...

✓ 找到 20 个文件 (共扫描 168 个文件, 12 个目录, 耗时 2341 ms)
```

**与全文搜索的区别：**

| 特性 | 全文搜索 (`--index`) | 语义搜索 (`--semantic`) |
|------|---------------------|------------------------|
| 匹配方式 | 关键词精确匹配 | 语义相似度 |
| 速度 | 极快 (p99 <1ms) | 较慢 (首次 >2s) |
| 准确性 | 高（关键词存在） | 中（理解能力有限） |
| 适用场景 | 已知关键词 | 概念性查询 |
| 索引持久化 | 支持 | 不支持（每次重建） |

**最佳实践：**

1. 先使用全文搜索，找不到时再用语义搜索
2. 限制搜索路径以提高速度
3. 使用文件类型过滤减少索引文件数
4. 查询尽量具体明确

## 常见问题

### Q: 前缀搜索没有找到预期结果？

**A:** 前缀搜索在分词后的 token 上进行，不是在原始文本上。

**示例：**

- "configuration" 可能被分为 "config" + "uration"
- 搜索 `config*` 可以匹配 "config" token
- 但不一定匹配完整的 "configuration"

**建议：**

- 使用标准搜索：`xore find "configuration" --index`
- 或使用更长的前缀：`xore find "configur*" --index`

### Q: 模糊搜索太慢？

**A:** 模糊搜索比精确搜索慢，因为需要计算编辑距离。

**优化方法：**

1. 减小搜索范围：`xore find "~term" --index --path ./src`
2. 使用文件类型过滤：`xore find "~term" --index --type log`
3. 考虑使用前缀搜索替代：`xore find "ter*" --index`

**性能数据：** 即使是模糊搜索，p99 延迟也仅 0.3ms，满足交互式使用需求。

### Q: 如何提高搜索速度？

**A:** 优化建议：

1. **确保索引是最新的**

   ```bash
   xore find "query" --index --rebuild
   ```

2. **使用 SSD 存储索引**
   - 索引默认存储在 `.xore/index`
   - 可通过 `--index-dir` 指定到 SSD 路径

3. **增加系统内存**
   - 更多内存可提升索引构建速度
   - 建议至少 8GB RAM

4. **使用文件类型过滤减少搜索范围**

   ```bash
   xore find "query" --index --type log
   ```

5. **启用 mimalloc 分配器**

   ```bash
   cargo build --release --features mimalloc
   ```

### Q: 索引占用多少空间？

**A:** 索引大小约为原始数据的 15-20%。

**示例：**

- 原始数据：9.5 GB
- 索引大小：约 1.5-2 GB
- 压缩后可进一步减小

### Q: 如何清理索引？

**A:** 删除索引目录即可：

```bash
# 删除默认索引
rm -rf .xore/index

# 删除自定义索引
rm -rf /path/to/custom/index
```

下次搜索时会自动重建索引。

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

### 普通扫描模式

```
扫描文件中...

   2.39 KB  ./src/main.rs
   4.40 KB  ./src/lib.rs
  10.23 KB  ./src/scanner.rs

✓ 找到 3 个文件 (共扫描 45 个文件, 12 个目录, 耗时 5 ms)
  总大小: 17.02 KB
  已跳过: 42 个文件 (不匹配过滤条件)
```

### 全文索引搜索模式 (`--index`)

首次搜索（构建索引）：

```
📑 构建索引中...
✓ 索引构建完成: 150 个文档 (扫描 200 个文件, 0 个错误, 耗时 2.35s)
🔍 搜索 "error"...

0.88 /path/to/error.log:1
    This is an error message

0.65 /path/to/app.rs:42
    fn handle_error(e: Error) -> Result<()>

✓ 找到 2 个匹配 (索引包含 150 个文档, 耗时 152.30ms)
```

后续搜索（使用已有索引）：

```
🔍 搜索 "错误"...

0.92 /path/to/chinese.log:5
    这是一个错误日志

✓ 找到 1 个匹配 (索引包含 150 个文档, 耗时 85.20ms)
```

## 相关命令

- [process](./process.md) - 处理找到的数据文件
- [benchmark](./benchmark.md) - 测试扫描性能

## 另请参阅

- [过滤器语法参考](../reference/filters.md)
