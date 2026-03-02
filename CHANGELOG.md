# Changelog

所有重要变更都会记录在此文件中。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)，
本项目遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

## [Unreleased]

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

- **Day 15: Polars 数据处理引擎集成** ([#day15](plans/day15-plan.md))
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
