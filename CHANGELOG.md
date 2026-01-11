# Changelog

所有重要变更都会记录在此文件中。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)，
本项目遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

## [Unreleased]

### Added
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
