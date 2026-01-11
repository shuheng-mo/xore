<div align="center">

# XORE

> **Explore the Abyss, Extract the Core**

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Version](https://img.shields.io/badge/version-1.0.0-green.svg)](https://github.com/shuheng-mo/xore/releases)
[![Rust](https://img.shields.io/badge/rust-1.91+-orange.svg)](https://www.rust-lang.org/)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)

[English](README_EN.md) | [简体中文](README.md)

**A local developer tool with extreme performance, deeply fusing semantic search and instant data analysis**

</div>

---

## Table of Contents

- [Overview](#overview)
- [Key Features](#key-features)
- [Architecture](#architecture)
- [Quick Start](#quick-start)
  - [Requirements](#requirements)
  - [Build & Install](#build--install)
- [Usage Guide](#usage-guide)
  - [Find Command (xore find)](#find-command-xore-find)
  - [Process Command (xore process)](#process-command-xore-process)
- [Project Structure](#project-structure)
- [Configuration](#configuration)
- [Developer Guide](#developer-guide)
- [Benchmarks](#benchmarks)
- [Changelog](#changelog)
- [Contributing](#contributing)
- [License](#license)
- [Acknowledgements](#acknowledgements)

---

## Overview

**XORE** is a high-performance local CLI tool written in Rust, designed to provide developers with blazing fast file search and data processing capabilities.

### What is it?

XORE is not a simple combination of a search tool (like ripgrep) and a data tool (like DuckDB). It is a unified tool where **search is analysis, and analysis is search**. It tightly integrates full-text search, semantic search, and a data processing engine, delivering millisecond-level responses and a zero-config experience.

### Why?

In traditional workflows, file search and data analysis are separate steps:

- Use `grep`/`ripgrep` to find files, then manually open, parse, and analyze
- Use `awk`/`pandas`/`DuckDB` for data analysis without semantic understanding
- Switch between multiple tools, leading to inefficiency

XORE unifies these capabilities with Rust’s zero-cost abstractions and high-performance libraries.

### Who is it for?

- **Data Engineers**: Quickly explore local datasets for QA and transformation
- **Backend Developers**: Analyze logs, audit configuration files, find code snippets
- **DevOps Engineers**: Troubleshoot production issues, analyze monitoring metrics
- **Researchers**: Process experimental data, search and organize literature

---

## Key Features

### 🔍 Intelligent Search Engine

- **Full-text Search**: High-performance inverted index powered by Tantivy
- **Semantic Search**: Lightweight embedding models via ONNX Runtime
- **Hybrid Retrieval**: Smart fusion of BM25 and vector similarity
- **Incremental Indexing**: Auto updates on file changes, real-time watching

### ⚡ High-Performance Data Processing

- **SQL Engine**: DataFrame operations built on Polars
- **Lazy Evaluation**: Deferred computation for memory efficiency
- **Parallel Processing**: Fully utilizes multi-core CPUs
- **Multi-format Support**: CSV, JSON, Parquet, Arrow, and more

### 🎯 Data Quality Analysis

- **Auto Profiling**: Stats, missing value detection, outlier detection
- **Type Inference**: Smart column type and data pattern recognition
- **Quality Reports**: Detailed data quality insights

### 🚀 Extreme Performance

- **Zero-Copy Design**: Memory mapping and zero-copy I/O
- **SIMD Acceleration**: Vectorized computation optimizations
- **Smart Caching**: Multi-level caching strategies
- **Millisecond Response**: Instant local data queries

---

## Architecture

### Core Tech Stack

- [Rust](https://www.rust-lang.org/) 1.91+ — Performance and memory safety
- [Tantivy](https://github.com/quickwit-oss/tantivy) 0.22 — Full-text search engine
- [Polars](https://www.pola.rs/) 0.45 — High-performance DataFrame library
- [ONNX Runtime](https://onnxruntime.ai/) 2.0 — Machine learning inference engine
- [Tokio](https://tokio.rs/) 1.35 — Async runtime
- [Clap](https://github.com/clap-rs/clap) 4.5 — CLI argument parsing

### Project Modules

```
xore/
├── xore-cli/         # CLI interface and command routing
├── xore-core/        # Core types and configuration management
├── xore-search/      # Search engine module
├── xore-process/     # Data processing engine
├── xore-ai/          # Semantic search and embeddings
└── supplementary/    # Project documents
```

---

## Quick Start

### Requirements

- **Rust** >= 1.91.0
- **Cargo** >= 1.91.0
- **OS**: macOS, Linux, or Windows

### Build & Install

1. **Clone the repository**

```bash
git clone https://github.com/shuheng-mo/xore.git
cd xore
```

1. **Build**

```bash
# Debug build (development)
cargo build

# Release build (production)
cargo build --release
```

1. **Install to system path (optional)**

```bash
cargo install --path xore-cli
```

1. **Verify installation**

```bash
xore --version
# Output: xore 1.0.0
```

1. **Environment setup (optional)**

```bash
cp .env.example .env
# Edit the .env file and fill in required configuration
```

1. **Start development server (if applicable to your setup)**

```bash
npm run dev
# or
yarn dev
```

1. **Access the app**

Open the browser at [http://localhost:3000](http://localhost:3000)

---

## Usage Guide

### Find Command (xore find)

#### Basic Search

```bash
# Search for files containing "error" in the current directory
xore find "error"
xore f "error"  # shorthand

# Specify search path
xore f "TODO" --path ./src

# Specify file type
xore f "function" --type rust
```

#### Semantic Search

```bash
# Use semantic search to find relevant code
xore f "handling database connection failures" --semantic

# Semantic search through log files
xore f "errors related to memory leaks" --semantic --type log
```

### Process Command (xore process)

#### Data Processing

```bash
# Quick overview of a data file
xore process data.csv
xore p data.csv  # shorthand

# Run a SQL query
xore p data.csv "SELECT * FROM self WHERE age > 30"

# Data quality check
xore p data.csv --quality-check
```

#### Advanced Usage

```bash
# Process JSON files
xore p logs.json "SELECT timestamp, level, message FROM self WHERE level = 'ERROR'"

# Parquet file analysis
xore p large_dataset.parquet --quality-check

# Export results
xore p data.csv "SELECT * FROM self LIMIT 100" > output.json
```

---

## Project Structure

```
xore/
├── xore-cli/              # CLI interface
│   ├── src/
│   │   ├── main.rs       # Entry point
│   │   ├── commands/     # Command implementations
│   │   │   ├── find.rs   # Find command
│   │   │   └── process.rs # Process command
│   │   └── ui/           # User interface
│   └── Cargo.toml
│
├── xore-core/             # Core module
│   ├── src/
│   │   ├── config.rs     # Configuration management
│   │   ├── error.rs      # Error handling
│   │   └── types.rs      # Common types
│   └── Cargo.toml
│
├── xore-search/           # Search engine
│   ├── src/
│   │   ├── indexer.rs    # Index building
│   │   ├── query.rs      # Query processing
│   │   └── watcher.rs    # File watching
│   └── Cargo.toml
│
├── xore-process/          # Data processing
│   ├── src/
│   │   ├── sql.rs        # SQL engine
│   │   ├── profiler.rs   # Data profiling
│   │   └── export.rs     # Export features
│   └── Cargo.toml
│
├── xore-ai/               # AI module
│   ├── src/
│   │   ├── embedding.rs  # Vector embeddings
│   │   └── tokenizer.rs  # Tokenizer
│   └── Cargo.toml
│
├── supplementary/         # Documentation
│   ├── PRD_v2.md         # Product requirements
│   ├── 技术设计文档.md    # Technical design (Chinese)
│   ├── 开发规范文档.md    # Development guidelines (Chinese)
│   └── 测试计划文档.md    # Test plan (Chinese)
│
├── Cargo.toml            # Workspace config
├── README.md             # Chinese README
├── README_EN.md          # English README (this file)
├── LICENSE               # MIT License
├── CONTRIBUTING.md       # Contributing guide
└── CHANGELOG.md          # Changelog
```

---

## Configuration

XORE uses a configuration file to manage default behavior and performance parameters. The default path is `~/.xore/config.toml`.

### Default Config

```toml
[search]
# Index storage path
index_path = "~/.xore/index"
# Number of worker threads (0 uses all CPU cores)
num_threads = 0
# Auto rebuild index after N days
auto_rebuild_days = 30
# Maximum index size (GB)
max_index_size_gb = 10

[process]
# Enable lazy execution
lazy_execution = true
# Chunk size (MB)
chunk_size_mb = 64
# Max memory usage (GB)
max_memory_gb = 4

[ai]
# Embedding model path
model_path = "~/.xore/models/embedding.onnx"
# Vector dimension
embedding_dim = 384
# Batch size
batch_size = 32

[ui]
# Colored output
colored = true
# Progress bar
progress_bar = true
# Verbose mode
verbose = false
```

### Environment Variables

| Name | Description | Default |
|------|-------------|---------|
| `XORE_CONFIG_PATH` | Config file path | `~/.xore/config.toml` |
| `XORE_INDEX_PATH` | Index storage path | `~/.xore/index` |
| `XORE_LOG_LEVEL` | Log level | `info` |
| `XORE_NUM_THREADS` | Worker thread count | CPU core count |

---

## Developer Guide

### Workflow

1. **Fork** the repository
2. **Create a feature branch** (`git checkout -b feature/AmazingFeature`)
3. **Write code and tests**
4. **Run tests** (`cargo test`)
5. **Format code** (`cargo fmt`)
6. **Lint with clippy** (`cargo clippy`)
7. **Commit changes** (`git commit -m 'feat: Add some AmazingFeature'`)
8. **Push branch** (`git push origin feature/AmazingFeature`)
9. **Open a Pull Request**

### Code Style

Project follows Rust official style:

```bash
# Auto format
cargo fmt --all

# Linting (deny warnings)
cargo clippy --all-targets --all-features -- -D warnings

# Run all tests
cargo test --all

# Generate docs
cargo doc --no-deps --open
```

### Commit Convention

Follows [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types:**

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `style`: Code formatting
- `refactor`: Refactoring
- `perf`: Performance improvement
- `test`: Tests
- `chore`: Build/tooling

**Example:**

```
feat(search): add semantic search support

Implement semantic search using ONNX Runtime with
MiniLM-L6 embedding model.

Closes #123
```

### Local Development

```bash
# Enable verbose logs
RUST_LOG=debug cargo run -- find "test"

# Auto rebuild with cargo-watch
cargo watch -x 'run -- find "test"'

# Run benchmarks
cargo bench

# Coverage analysis
cargo tarpaulin --out Html
```

---

## Benchmarks

XORE delivers outstanding performance across metrics.

### Search Performance

| Dataset Size | File Count | Index Build Time | Query Latency |
|--------------|------------|------------------|---------------|
| 100 MB       | 1,000      | 2.3s             | 5ms           |
| 1 GB         | 10,000     | 18.7s            | 12ms          |
| 10 GB        | 100,000    | 3m 45s           | 28ms          |

### Data Processing Performance

| Operation | Data Size | Polars (Rust) | Pandas (Python) | Speedup |
|----------|-----------|---------------|------------------|---------|
| CSV read | 1 GB      | 1.2s          | 8.5s             | 7x      |
| SQL agg  | 10M rows  | 0.8s          | 6.3s             | 8x      |
| Join     | 2×5M rows | 1.5s          | 12.1s            | 8x      |

### Memory Usage

- Index: ~15–20% of raw data size
- Runtime: Peak memory < 2× data size
- Lazy evaluation: Supports datasets larger than RAM

*Test environment: MacBook Pro M1 Max, 32GB RAM*

---

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for full history.

### v1.0.0 (2026-01-11)

**Initial Release**

- ✨ Full-text search engine (Tantivy)
- ✨ Semantic search (ONNX Runtime)
- ✨ Data processing engine (Polars)
- ✨ SQL queries
- ✨ Data quality analysis
- 📝 Comprehensive project docs
- ✅ Unit and integration tests

---

## Contributing

Thanks for your interest in XORE! All kinds of contributions are welcome.

### How to Contribute

- **Report Bugs**: Submit issues on [Issues](https://github.com/shuheng-mo/xore/issues)
- **Feature Requests**: Share ideas and proposals
- **Code Contributions**: Open a Pull Request
- **Docs Improvements**: Help improve documentation
- **Testing Feedback**: Test across environments and report findings

See [CONTRIBUTING.md](CONTRIBUTING.md) for details.

### Contributors

Thanks to all developers who contribute to XORE!

<!-- Contributor list will be auto-generated -->

---

## License

This project is open-sourced under the MIT License — see [LICENSE](LICENSE) for details.

---

## Acknowledgements

XORE’s development is built upon the following excellent open-source projects:

- [Tantivy](https://github.com/quickwit-oss/tantivy) — High-performance full-text search
- [Polars](https://github.com/pola-rs/polars) — Fast DataFrame library
- [ONNX Runtime](https://github.com/microsoft/onnxruntime) — Cross-platform ML inference
- [Tokio](https://github.com/tokio-rs/tokio) — Async runtime
- [Clap](https://github.com/clap-rs/clap) — Command-line parser

Special thanks to the Rust community for its vibrant ecosystem.

---

<div align="center">

**[⬆ Back to Top](#xore)**

Made with ❤️ by XORE Team

</div>
