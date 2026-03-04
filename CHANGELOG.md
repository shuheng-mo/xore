# Changelog

所有重要变更都会记录在此文件中。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)，
本项目遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

## [Unreleased]

### Added

- **Day 24: 语义搜索 CLI 集成** ([#day24](plans/day24-semantic-search-plan.md))
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

- **Day 22-23: ONNX 集成与语义搜索基础** ([#day22-23](plans/day22-23-onnx-plan.md))
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
- **修复 Watch 模式增量索引功能** ([#watch-mode-fix](plans/fix-watch-mode.md))
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

- **Day 20-21: SIMD 优化与数据导出功能** ([#day20-21](plans/day20-21-plan.md))
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

- **Day 19: 数据质量检测增强**
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

- **Day 17-18: SQL 查询引擎实现** ([#day17-18](plans/day17-18-plan.md))
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

- **Day 15-16: Polars 数据处理引擎集成** ([#day15](plans/day15-plan.md))
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

[Unreleased]: https://github.com/yourusername/xore/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/yourusername/xore/releases/tag/v1.0.0
