<table>
<tr>
<td width="140" valign="middle">
<img src="assets/xore.png" alt="XORE" width="112" />
</td>
<td valign="middle">
<h1>XORE</h1>
<p><em>Explore the Abyss, Extract the Core</em></p>
<p>
<a href="LICENSE"><img src="https://img.shields.io/badge/license-GPL--3.0-blue.svg" alt="License" /></a>
<a href="https://github.com/shuheng-mo/xore/releases"><img src="https://img.shields.io/badge/version-1.0.0-green.svg" alt="Version" /></a>
<a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.91+-orange.svg" alt="Rust" /></a>
<a href="https://github.com/shuheng-mo/xore/actions/workflows/ci.yml"><img src="https://github.com/shuheng-mo/xore/actions/workflows/ci.yml/badge.svg" alt="CI" /></a>
<a href="CONTRIBUTING.md"><img src="https://img.shields.io/badge/PRs-welcome-brightgreen.svg" alt="PRs Welcome" /></a>
</p>
<p><a href="README_EN.md">English</a> | <a href="README.md">简体中文</a></p>
</td>
</tr>
</table>

**一站式极致性能本地开发者工具，将语义搜索与数据即时分析深度融合，毫秒级响应，零配置体验，专为开发者与 AI Agents 深度优化。**

---

## 目录

- [项目简介](#项目简介)
- [核心特性](#核心特性)
- [技术架构](#技术架构)
- [快速开始](#快速开始)
  - [环境要求](#环境要求)
  - [编译安装](#编译安装)
- [项目结构](#项目结构)
- [配置说明](#配置说明)
- [开发指南](#开发指南)
- [性能基准](#性能基准)
- [更新日志](#更新日志)
- [贡献指南](#贡献指南)
- [许可证](#许可证)
- [致谢](#致谢)

---

## 项目简介

**XORE** 是一款使用 Rust 开发的高性能本地 CLI 工具，旨在为开发者提供极速的文件搜索和数据处理能力。

### 这是什么？

XORE 不是简单的搜索工具（如 ripgrep）+ 数据工具（如 DuckDB）的组合，而是一个让**搜索即分析、分析即搜索**的一体化工具。它将全文搜索、语义搜索和数据处理引擎深度整合，提供极简配置体验。

### 为什么做这个？

传统的开发工作流中，文件搜索和数据分析是两个独立的环节：

- 用 `grep`/`ripgrep` 找到文件后，还需要手动打开、解析、分析
- 用 `awk`/`pandas`/`DuckDB` 分析数据时，缺乏语义理解能力
- 需要在多个工具间切换，效率低下，这点在Coding Agent工作流中尤为明显，反复切换工具和上下文，导致效率大幅下降，token用量大量增加。

XORE 通过 Rust 的零成本抽象和高性能库，将这些能力统一到一个工具中，提升开发者和 Agent 的效率。

### 适用场景

- **数据工程师**：快速探索本地数据集，进行质量检查和转换
- **后端开发者**：分析日志文件，审计配置文件，查找代码片段
- **DevOps 工程师**：排查生产问题，监控指标分析
- **研究人员**：处理实验数据，文献检索和整理
- **AI Agents**：提供 Agent 友好的接口，支持结构化摘要和智能采样，极大节省 Token 用量和对话轮次。

---

## 核心特性

### 🤖 针对Agent优化 (`xore agent`)

- **结构化摘要**：`xore agent schema` 仅返回数据结构与分布，不搬运原始数据。
- **智能采样**：`xore agent sample` 自动提取最具代表性的数据样本。
- **Token 预算控制**：自动对长文本进行语义压缩，保留核心逻辑（如函数头/尾），中略冗余实现。
- **Agent 修复建议**：报错信息自动转化为可执行的修复指令。

### 🔍 智能搜索引擎

- **语义块切片**：基于 Tree-sitter 感知代码结构，返回完整的函数/类块而非单纯的行。
- **全文搜索**：基于 Tantivy 的高性能倒排索引，BM25 排序。
- **模糊与前缀**：支持 `~term` 模糊匹配与 `term*` 前缀搜索。
- **增量索引**：毫秒级文件监控更新（`--watch`），确保 Agent 看到的是最新状态。

### ⚡ 高性能数据处理

- **计算下推**：在本地利用 Polars 引擎完成 SQL 过滤、聚合，仅输出结果，避免向 Agent 传输原始数据。
- **零拷贝读取**：大文件（>1MB）自动使用 `memmap2` 内存映射，GB 级数据秒级加载。
- **惰性求值**：LazyFrame 延迟执行优化内存占用，支持处理远超内存大小的超大规模数据集。
- **多格式支持**：原生支持 CSV, JSON, Parquet, Arrow 等，自动 Schema 推断。

### 🎯 数据质量分析

- **自动 Profiling**：统计分析、缺失值检测（百分比）、重复行识别。
- **类型推断**：智能识别列类型和数据模式，自动推断 Schema。
- **离群值检测**：IQR 方法检测数值列异常值（API 可用）。
- **列统计**：唯一值数量、缺失值分布、数据类型信息。

### 🚀 极致性能

- **零拷贝设计**：内存映射和零拷贝 I/O
- **SIMD 加速**：向量化计算优化
- **智能缓存**：多级缓存策略
- **毫秒级响应**：本地数据即时查询

---

## 技术架构

### 核心技术栈

- [Rust](https://www.rust-lang.org/) 1.91+ - 系统编程语言，保证性能和内存安全
- [Tantivy](https://github.com/quickwit-oss/tantivy) 0.22 - 全文搜索引擎
- [Polars](https://www.pola.rs/) 0.45 - 高性能 DataFrame 库
- [ONNX Runtime](https://onnxruntime.ai/) 2.0 - 机器学习推理引擎
- [Tokio](https://tokio.rs/) 1.35 - 异步运行时
- [Clap](https://github.com/clap-rs/clap) 4.5 - 命令行参数解析

### 项目模块

```
xore/
├── xore-cli/         # CLI 界面和命令路由
├── xore-core/        # 核心类型和配置管理
├── xore-search/      # 搜索引擎模块
├── xore-process/     # 数据处理引擎
├── xore-ai/          # 语义搜索和嵌入
└── docs/    # 项目文档
```

---

## 快速开始

### 环境要求

- **Rust** >= 1.91.0
- **Cargo** >= 1.91.0
- **操作系统**：macOS、Linux 或 Windows

### 编译安装

#### 快速安装（推荐）

```bash
# 方式一：一键安装脚本（macOS/Linux）
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/shuheng-mo/xore/releases/latest/download/xore-installer.sh | sh

# 方式二：使用 cargo-binstall
cargo binstall xore

# 方式三：源码编译安装
cargo install --path xore-cli
```

#### 手动编译

- **克隆项目**

```bash
git clone https://github.com/shuheng-mo/xore.git
cd xore
```

- **编译**

```bash
# Debug 构建（开发调试）
cargo build

# Release 构建（生产使用）
cargo build --release
```

- **安装到系统路径（可选）**

```bash
cargo install --path xore-cli
```

- **验证安装**

```bash
xore --version
# 输出：xore 1.0.0
```

---

## 详细使用文档

请参阅 [docs/README.md](docs/README.md) 获取完整的使用指南。

---

## 项目结构

```
xore/
├── xore-cli/              # CLI 命令行界面
├── xore-config/           # 配置管理模块
├── xore-core/             # 核心共享模块
├── xore-search/           # 搜索引擎
├── xore-process/          # 数据处理引擎
├── xore-ai/               # AI 语义搜索模块
├── docs/                  # 项目文档
└── examples/              # 示例数据和脚本
```

---

## 配置说明

XORE 使用配置文件来管理默认行为和性能参数。配置文件位于 `~/.xore/config.toml`。

### 默认配置

```toml
[search]
# 索引存储路径
index_path = "~/.xore/index"
# 工作线程数（默认使用所有 CPU 核心）
num_threads = 0
# 自动重建索引的天数
auto_rebuild_days = 30
# 最大索引大小（GB）
max_index_size_gb = 10

[process]
# 启用惰性求值
lazy_execution = true
# 分块大小（MB）
chunk_size_mb = 64
# 最大内存使用（GB）
max_memory_gb = 4

[ai]
# 嵌入模型路径
model_path = "~/.xore/models/embedding.onnx"
# 向量维度
embedding_dim = 384
# 批处理大小
batch_size = 32

[ui]
# 彩色输出
colored = true
# 进度条
progress_bar = true
# 详细模式
verbose = false
```

### 环境变量

| 变量名 | 说明 | 默认值 |
|--------|------|--------|
| `XORE_CONFIG_PATH` | 配置文件路径 | `~/.xore/config.toml` |
| `XORE_INDEX_PATH` | 索引存储路径 | `~/.xore/index` |
| `XORE_LOG_LEVEL` | 日志级别 | `info` |
| `XORE_NUM_THREADS` | 工作线程数 | CPU 核心数 |

---

## 开发指南

### 开发流程

1. **Fork 项目仓库**
2. **创建特性分支** (`git checkout -b feature/AmazingFeature`)
3. **编写代码和测试**
4. **运行测试** (`cargo test`)
5. **代码格式化** (`cargo fmt`)
6. **代码检查** (`cargo clippy`)
7. **提交更改** (`git commit -m 'feat: Add some AmazingFeature'`)
8. **推送到分支** (`git push origin feature/AmazingFeature`)
9. **提交 Pull Request**

### 代码规范

项目遵循 Rust 官方代码风格：

```bash
# 自动格式化代码
cargo fmt --all

# 代码质量检查
cargo clippy --all-targets --all-features -- -D warnings

# 运行所有测试
cargo test --all

# 生成文档
cargo doc --no-deps --open
```

### 提交规范

遵循 [Conventional Commits](https://www.conventionalcommits.org/) 规范：

```
<type>(<scope>): <subject>

<body>

<footer>
```

**类型（Type）：**

- `feat`: 新功能
- `fix`: 修复 Bug
- `docs`: 文档更新
- `style`: 代码格式调整
- `refactor`: 代码重构
- `perf`: 性能优化
- `test`: 测试相关
- `chore`: 构建/工具链更新

**示例：**

```
feat(search): add semantic search support

Implement semantic search using ONNX Runtime with
MiniLM-L6 embedding model.

Closes #123
```

### 本地开发

```bash
# 启用详细日志
RUST_LOG=debug cargo run -- find "test"

# 使用 cargo-watch 自动重新编译
cargo watch -x 'run -- find "test"'

# 运行基准测试
cargo bench

# 分析代码覆盖率
cargo tarpaulin --out Html
```

---

## 性能基准

基于最新测试数据集的实际测试结果（2026-03-05）：

### 搜索性能对比

| 场景 | ripgrep | XORE | 胜者 |
|------|---------|------|------|
| 小型日志搜索 (500行) | 14ms | 3ms | ✅ XORE |
| 中型日志搜索 (200K行) | 13ms | 2ms | ✅ XORE |

### 数据处理性能对比

| 场景 | DuckDB | Pandas | XORE | 胜者 |
|------|--------|--------|------|------|
| 小型 CSV (100行) | 135ms | - | 5ms | ✅ XORE |
| 小型 JSON (100行) | - | - | 2ms | ✅ XORE |
| 小型 Parquet (100行) | 23ms | - | 2ms | ✅ XORE |
| 中型 CSV COUNT (100K行) | 433ms | - | 43ms | ✅ XORE |
| 中型 CSV GROUP BY | 156ms | - | 34ms | ✅ XORE |
| 中型 CSV WHERE | 126ms | - | 26ms | ✅ XORE |
| 中型 Parquet COUNT | 21ms | - | 9ms | ✅ XORE |
| 大型 CSV (600MB, ~10M行) | 501ms | 8060ms | 1268ms | ❌ ripgrep |

### 性能优势

| 对比项 | 传统工具 | XORE | 优势 |
|-------|---------|------|------|
| **Token 效率** | 原始文本搬运 | **计算下推/结构化摘要** | **节省 90%+ Token** |
| **全文搜索** | ripgrep (线性扫描) | 索引加速 | 5x+ |
| **数据处理** | DuckDB/Pandas | Polars 引擎 | 3-10x |
| **大文件处理** | 内存加载 | 零拷贝 mmap | 内存节省 90%+ |

### 内存占用

- 索引: 约为原始数据的 15-20%
- 大文件: 使用 mmap 零拷贝，内存占用接近于零
- 运行时: 峰值内存 < 数据大小的 2 倍

*测试环境：macOS (Apple Silicon), 对比 ripgrep 15.1.0, DuckDB v1.4.4*

---

## 更新日志

查看 [CHANGELOG.md](CHANGELOG.md) 了解详细的版本历史。

### v1.0.0 (2026-01-11)

**初始版本发布**

- ✨ 实现全文搜索引擎（基于 Tantivy）
- ✨ 实现语义搜索（基于 ONNX Runtime）
- ✨ 实现数据处理引擎（基于 Polars）
- ✨ 支持 SQL 查询
- ✨ 数据质量分析功能
- 📝 完善项目文档
- ✅ 添加单元测试和集成测试

---

## 贡献指南

感谢你对 XORE 项目的关注！我们欢迎各种形式的贡献。

### 如何贡献

- **报告 Bug**：在 [Issues](https://github.com/shuheng-mo/xore/issues) 页面提交问题
- **功能建议**：提出新功能的想法和建议
- **代码贡献**：提交 Pull Request
- **文档改进**：帮助改进文档质量
- **测试反馈**：在不同环境下测试并反馈问题

详细的贡献指南请查看 [CONTRIBUTING.md](CONTRIBUTING.md)。

### 贡献者

感谢所有为 XORE 做出贡献的开发者！

<!-- 贡献者列表将自动生成 -->

---

## 许可证

查看 [LICENSE](LICENSE) 文件了解详情。

---

## 致谢

XORE 的开发离不开以下优秀的开源项目：

- [Tantivy](https://github.com/quickwit-oss/tantivy) - 高性能全文搜索引擎
- [Polars](https://github.com/pola-rs/polars) - 快速的 DataFrame 库
- [ONNX Runtime](https://github.com/microsoft/onnxruntime) - 跨平台 ML 推理引擎
- [Tokio](https://github.com/tokio-rs/tokio) - 异步运行时
- [Clap](https://github.com/clap-rs/clap) - 命令行参数解析库

特别感谢 Rust 社区提供的强大生态系统。

---

<div align="center">

**[⬆ 回到顶部](#xore)**

Made with ❤️ by XORE Team

</div>
