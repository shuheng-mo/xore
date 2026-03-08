# Changelog

所有重要变更都会记录在此文件中。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)，
本项目遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

## [1.2.0] (2026-03-08)

### Added

- **xore agent peek 命令 - 智能目录预览**
  - 实现目录扫描和智能预览功能（`xore-cli/src/commands/peek.rs`）
  - 支持多种输出格式：JSON、树形结构、Markdown
  - 文件类型智能识别：代码文件、配置文件、文本文件、二进制文件
  - 代码结构提取：支持 Rust/Python/JavaScript 的函数和类提取
  - 内存缓存机制：使用 `once_cell::Lazy` 实现带 TTL 的缓存
  - 配置选项：
    - `--depth` 目录遍历深度（默认 3 层）
    - `--format` 输出格式（json/tree/md）
    - `--include/--exclude` 文件过滤规则
    - `--code-structure` 提取代码结构
  - **测试覆盖**：8 个单元测试全部通过

- **Watch 守护进程模式**
  - 实现后台守护进程管理（`xore-cli/src/commands/watch.rs`）
  - 支持守护进程启动、停止、状态查看、日志查看
  - 使用 `setsid()` 实现进程脱离终端
  - PID 文件管理：`~/.xore/run/watch.pid`
  - 日志轮转：自动轮转和清理旧日志
  - 子命令：
    - `xore watch start` - 启动守护进程
    - `xore watch stop` - 停止守护进程
    - `xore watch status` - 查看状态
    - `xore watch logs` - 查看日志
  - **测试覆盖**：6 个单元测试全部通过

- **xore agent abyss 全局文件监控**
  - 实现全局文件监控守护进程（`xore-cli/src/commands/abyss.rs`）
  - 实时监控用户主目录下的文件变化
  - 安全机制：
    - 首次启动显示隐私警告，需要用户确认
    - 检查系统权限（Linux inotify/macOS FSEvents）
    - 可配置排除目录和文件类型
  - 环境变量配置：
    - `XORE_ABYSS_EXCLUDE` - 排除目录列表
    - `XORE_ABYSS_INCLUDE` - 包含文件扩展名
  - 子命令：
    - `xore agent abyss start` - 启动监控
    - `xore agent abyss stop` - 停止监控
    - `xore agent abyss status` - 查看状态
    - `xore agent abyss logs` - 查看日志
    - `xore agent abyss stats` - 查看统计信息
    - `xore agent abyss config` - 查看/更新配置
  - **测试覆盖**：10 个单元测试全部通过

- **Find 命令增强**
  - 新增 `--watch-daemon` 参数，支持启动 Watch 守护进程后自动退出
  - 新增 `--watch-include/--watch-exclude` 参数，配置 Watch 过滤规则

- **配置模块增强**
  - 新增 `PeekConfig` 配置结构（`xore-config/src/config.rs`）
  - 新增 `WatchConfig` 配置结构
  - 新增 `AbyssConfig` 配置结构

### Changed

- **依赖更新**
  - 添加 `chrono` 0.4（时间处理）
  - 添加 `dirs` 5.0（目录路径获取）
  - 添加 `once_cell` 1.19（静态初始化）

### Fixed

- 修复测试并发问题：环境变量测试添加 `ENV_MUTEX` 互斥锁
- 修复 PID 溢出问题：使用真实进程而非 `u32::MAX`
- 修复 Clippy 警告：`filter_map` 改为 `map_while`

## [1.1.0] (2026-03-07)

### Added

- **MCP 服务器支持**
  - 新增 `xore-mcp` crate，提供 Model Context Protocol 服务器
  - 集成到 Roo Code 等 AI 助手，实现文件搜索和数据处理能力
  - 提供 7 个 MCP 工具：`find_files`, `search_index`, `get_schema`, `query_data`, `sample_data`, `quality_check`, `get_config`
  - 详细文档见 [docs/mcp.md](docs/mcp.md)

- **错误处理优化 (修复)**
  - **Bug 修复**：修复 `--verbose` 标志在子命令上不工作的问题
    - 原因：`verbose` 参数未设置为全局标志 (`global = true`)
    - 修复：在 `xore-cli/src/main.rs` 中添加 `global = true` 属性
    - 现在支持：`xore find --verbose "query"` 和 `xore --verbose find "query"` 两种用法

- **错误处理优化**
  - **扩展 `XoreError` 枚举** (`xore-core/src/error/mod.rs`)：
    - 新增错误类型：`SearchError`, `ParseError`, `ValidationError`, `Timeout`, `PermissionDenied`
    - 新增 `error_code()` 方法，返回机器可读的错误代码字符串
    - 新增 `XoreErrorExt` trait，提供 `context()`, `with_location()`, `hint()` 方法
    - 新增 `ErrorContext` 结构，支持多条上下文消息和位置信息
    - 新增 `ErrorHint` 结构，支持智能提示、建议命令和文档链接
    - 新增 `ErrorChain` 结构，支持错误来源追踪
  - **统一错误格式化器** (`xore-core/src/error/format.rs`)：
    - 实现 `ErrorFormatter`，参考 Rust 编译器风格输出
    - 支持彩色/无彩色模式（`use_color` 配置）
    - 支持 `--verbose` 模式显示详细堆栈和解决方案
    - 支持智能提示开关（`show_hints` 配置）
    - 提供 `print_error()` 和 `print_anyhow_error()` CLI 辅助函数
  - **CLI 集成** (`xore-cli/src/main.rs`)：
    - 重构 `main()` 函数，统一错误输出格式
    - 错误时调用 `print_anyhow_error()` 格式化输出
    - `--verbose` 模式显示完整错误链
    - 错误退出码 `std::process::exit(1)`
  - **各模块错误上下文优化**：
    - `xore-process/src/parser.rs`：文件不存在、CSV/Parquet 读取失败均附带友好提示
    - `xore-process/src/sql.rs`：SQL 执行失败附带 `xore agent explain` 建议
    - `xore-search/src/indexer.rs`：索引创建/打开/提交失败附带重建建议
    - `xore-search/src/query.rs`：查询解析/执行失败附带语法提示
  - **测试覆盖**：43 个错误处理测试全部通过（新增 28 个）

- **文档与示例完善**
  - README 文档完善：添加项目定位和性能数据
  - 命令文档完善：`docs/commands/` 下所有命令文档更新
  - 新增 `docs/commands/agent.md`: Agent 命令文档
  - 示例目录完善：`examples/benchmark-data/`, `examples/benchmark-results/`
  - 帮助信息完善：所有命令 `--help` 输出完整且清晰
  - Roo Code Skills 文档：`.roo/skills/*/SKILL.md` 完善
  - **MVP 开发完成总结**：27/28 天任务完成

- **Agent-Native 接口与 Roo Code Skills 集成**
  - **Agent 命令模块** (`xore-cli/src/commands/agent.rs`)：
    - 实现 `xore agent` 命令，提供 5 个子命令：
      - `xore agent init` - 生成 Agent 提示词模板（支持 Claude/GPT-4 等模型）
      - `xore agent schema` - 获取数据结构（零拷贝，不读取完整文件）
      - `xore agent sample` - 智能采样（random/head/tail/smart 四种策略）
      - `xore agent query` - SQL 查询并输出 JSON 格式
      - `xore agent explain` - SQL 错误分析与修复建议
    - **计算下推优化**：通过 schema 和 sample 减少 90%+ Token 消耗
    - **结构化输出**：JSON 格式便于 AI Agent 解析和处理
  - **Roo Code Skills 集成** (`.roo/skills/`)：
    - `xore-search` - 本地文件搜索 skill
    - `xore-data-analysis` - 数据分析 skill
    - `xore-agent` - Agent 优化 skill
    - 支持 VS Code AI 助手快速调用 XORE 功能
  - **依赖更新**：
    - 添加 `polars` 0.45（用于数据处理）
    - 添加 `rand` 0.8（用于随机采样）
  - **测试覆盖**：6 个单元测试全部通过

- **智能推荐系统**
  - **搜索历史模块** (`xore-core/src/history.rs`)：
    - 实现 `SearchType` 枚举：FullText, Semantic, FileType, SemanticWithFilter
    - 实现 `SearchHistoryEntry` 结构：记录查询、搜索类型、路径、时间戳、结果数、执行时间
    - 实现 `HistoryStore` 存储引擎：JSON 文件持久化，自动加载历史数据
    - 支持搜索统计：查询频率、平均结果数、平均执行时间
    - 自动创建 `~/.xore/history/` 目录
  - **推荐引擎模块** (`xore-core/src/recommendation.rs`)：
    - 实现 `RecommendationEngine` 智能推荐引擎
    - 基于搜索频率的推荐生成
    - 支持多种推荐类型：频繁查询、路径模式、文件类型模式
    - 置信度评分系统
  - **CLI 集成**：
    - `xore f "query"` 自动记录搜索历史
    - `xore f --history` 显示搜索历史
    - `xore f --recommend` 显示智能推荐
    - `xore f --clear-history` 清除搜索历史
  - **数据存储**：JSON 文件存储在 `~/.xore/history/history.json`
  - **测试覆盖**：16 个单元测试通过

- **语义搜索 CLI 集成**
  - **CLI 集成**：
    - 实现 `xore f --semantic` 语义搜索命令（`xore-cli/src/commands/find.rs`）
    - 支持环境变量配置模型路径（`XORE_MODEL_PATH`, `XORE_TOKENIZER_PATH`）
    - 自动文件内容读取和向量索引构建
    - 实时进度显示和相似度评分输出
  - **功能特性**：
    - 最多索引 1000 个文件（防止内存溢出）
    - 跳过大于 1MB 的文件
    - 跳过空文件和二进制文件
    - 返回 Top-20 相似结果
  - **文档更新**：
    - 更新 `docs/commands/find.md` 添加语义搜索章节
    - 包含使用示例、性能指标、最佳实践
    - 对比全文搜索与语义搜索的区别

- **ONNX 集成与语义搜索基础**
  - **ONNX Runtime 集成**：
    - 实现 `EmbeddingModel` 加载和推理（`xore-ai/src/embedding.rs`）
    - 支持 MiniLM-L6-v2 模型（384维向量）
    - 文本嵌入向量生成功能
    - L2 归一化和平均池化
  - **Tokenizer 封装**：
    - 基于 HuggingFace tokenizers 实现（`xore-ai/src/tokenizer.rs`）
    - 支持 WordPiece 分词
    - 批量编码优化
  - **向量搜索引擎**：
    - 实现 `VectorSearcher` 语义搜索（`xore-ai/src/search.rs`）
    - 余弦相似度计算
    - 文档索引管理
    - Top-K 搜索结果排序
  - **测试覆盖**：6 个单元测试通过
  - **文档**：完整的 README 和使用示例

### Fixed

- 修复 `is_binary_content()` 函数的UTF-8字符边界错误，避免在8000字节位置切割多字节字符时panic
- **修复 Watch 模式增量索引功能**
  - 修复 `execute_watch_mode()` 缺少事件循环调用的问题，现在能够正确处理文件变更事件
  - 修复 `IncrementalIndexer::commit()` 空实现问题，现在能够真正持久化索引变更
  - 添加 `IndexBuilder::commit_changes()` 方法，支持增量索引场景的多次提交
  - 使用 `tokio::select!` 优雅处理 Ctrl+C 信号，确保退出时提交最后的变更
  - 增强统计报告，每10秒显示创建/修改/删除的文件数和待提交变更数
  - 添加文件存在性检查，避免处理已删除文件时出错
  - **测试验证**：文件创建/修改/删除事件均能正确索引，搜索结果实时更新

### Changed

- **更新项目定位为 Agent-Native**
  - README 中强调 XORE 作为 AI Agent 的高性能工具，通过"计算下推"和"结构化摘要"降低 90%+ Token 消耗
  - 新增"核心差异化"章节，对比 XORE 与 ripgrep 等传统工具的优势
  - 更新性能基准，增加 Agent Efficiency (Token Savings) 指标

### Added

- **SIMD 优化与数据导出功能**
  - **SIMD 数值计算优化**：
    - 实现循环展开优化的数值计算函数（`xore-process/src/simd.rs`）
    - 提供 `sum_f64_simd`, `mean_f64_simd`, `variance_f64_simd`, `std_dev_f64_simd` 等函数
    - 提供 `min_f64_simd`, `max_f64_simd` 高性能查找函数
    - 使用 4 路循环展开技术提升性能 2-3x
    - **测试覆盖**：14 个单元测试全部通过
    - **基准测试**：新增 `xore-process/benches/simd_bench.rs`
  - **完整数据导出功能**：
    - 重写 `xore-process/src/export.rs`，支持 4 种导出格式
    - 支持格式：CSV, JSON (JSONL), Parquet, Arrow (使用 Parquet 替代)
    - 支持流式导出大文件（分块写入）
    - 支持自定义配置：缓冲区大小、分隔符、压缩类型
    - 支持导出到标准输出（管道模式）
    - **CLI 集成**：`xore p <file> "<sql>" -o output.csv`
    - **测试覆盖**：8 个单元测试全部通过
  - **性能优化**：
    - 数值计算性能提升 2-3x（循环展开）
    - 支持 GB 级文件导出，内存占用 <100MB
    - 自动格式检测（从文件扩展名推断）

- **数据质量检测增强**
  - 扩展 `QualityReport` 结构，增加智能建议和离群值信息
  - 实现智能建议生成系统：
    - 基于缺失值比例自动生成处理建议（Error/Warning/Info 三级严重程度）
    - 基于重复行数生成去重建议
    - 基于离群值检测生成数据异常提示
  - 优化离群值检测算法：
    - 支持批量检测所有数值列的离群值
    - 使用 IQR 方法（四分位距）检测异常值
    - 自动过滤非数值列，避免类型错误
  - 完善 CLI 输出格式：
    - 彩色高亮显示不同严重程度的问题（红色/黄色/正常）
    - 按严重程度排序显示建议（Error > Warning > Info）
    - 增加离群值检测结果展示
  - **测试覆盖**：新增 5 个单元测试，总计 9 个测试全部通过

- **SQL 查询引擎实现**
  - 实现基于 Polars `SQLContext` 的 SQL 查询引擎
  - 支持完整的 SQL 查询功能：
    - 基本查询：`SELECT`, `WHERE`, `ORDER BY`, `LIMIT`
    - 聚合查询：`GROUP BY`, `COUNT`, `SUM`, `AVG`, `MIN`, `MAX`
    - 多表查询：`INNER JOIN`, `LEFT JOIN`
  - CLI 集成：`xore p <file> "<sql>"` 命令支持
  - 自动表注册：使用文件名作为表名
  - 结果渲染：表格化输出，最多显示 100 行
  - **测试覆盖**：9 个单元测试全部通过
  - **性能优化**：LazyFrame 延迟执行 + 零拷贝读取

- **测试与基准**
  - 创建完整的基准测试数据集（`examples/benchmark-data/`）
  - 自动化基准测试脚本（`examples/benchmark-results/run_benchmark.sh`）
  - 测试覆盖：130+ 单元测试 + 30+ 集成测试全部通过
  - 代码质量检查：cargo fmt + clippy + check 通过

- **搜索优化**
  - 模糊匹配实现（Levenshtein 距离 <2）
  - 前缀搜索实现（基于 FST）
  - 文件类型权重调整（代码 > 文档 > 日志）
  - 批量索引优化和缓存优化

- **增量索引与文件监控**
  - 增量索引模块（`xore-search/src/incremental.rs`）
  - 文件监控模块（`xore-search/src/watcher.rs`，基于 notify crate）
  - 事件防抖（500ms）
  - CLI 集成：`xore f "error" --index --watch`
  - Bug 修复：Watch 模式增量索引功能

- **Tantivy 全文搜索引擎集成**
  - Tantivy 索引核心模块（`xore-search/src/indexer.rs`）
  - 查询引擎（`xore-search/src/query.rs`）
  - 自定义中英文分词器（`xore-search/src/tokenizer.rs`）
  - 支持 BM25 排序算法
  - 支持分页查询和高亮显示
  - 测试覆盖：20 个单元测试全部通过

- **Polars 数据处理引擎集成**
  - 实现 `DataParser` 模块，支持 CSV 和 Parquet 文件的高性能读取
  - 集成 `memmap2` 实现零拷贝读取，支持 GB 级大文件（阈值 1MB）
  - 实现 `LazyFrame` 模式，延迟执行优化内存占用
  - 实现 `DataProfiler` 模块，提供数据质量检测功能
  - 支持自动 Schema 推断和数据类型识别
  - 重构 `xore p` 命令，使用 Polars 替代手动字符串解析
  - 新增功能：
    - CSV/Parquet 数据预览（显示前 10 行）
    - 数据质量检查（缺失值、重复行检测）
    - 列统计信息（唯一值、缺失值百分比）
    - 离群值检测（IQR 方法）
  - **测试覆盖**：13 个单元测试全部通过
  - **性能验证**：成功读取和处理测试数据集

- 初始项目脚手架
- 基础CLI框架
- 核心模块结构

## [1.0.0] - TBD

### Added

- 全文搜索功能（基于Tantivy）
- 语义搜索功能（基于ONNX）
- 增量索引支持
- CSV/Parquet数据处理
- SQL查询引擎
- 数据质量检测
- 命令行界面

### Performance

- SIMD优化CSV解析
- 零拷贝数据管道
- 并发索引构建

[1.2.0]: https://github.com/shuheng-mo/xore/releases/tag/v1.2.0
[1.1.0]: https://github.com/shuheng-mo/xore/releases/tag/v1.1.0
[1.0.0]: https://github.com/shuheng-mo/xore/releases/tag/v1.0.0
