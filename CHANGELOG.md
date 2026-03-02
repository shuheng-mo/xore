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
