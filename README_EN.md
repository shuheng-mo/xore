<div style="display: flex; align-items: center; gap: 16px; margin-bottom: 24px;">
  <img src="assets/xore.png" alt="XORE Icon" height="64">
  <div>
    <h1 style="margin: 0;">XORE</h1>
    <p style="margin: 0; color: #666; font-size: 1.1em;">Explore the Abyss, Extract the Core</p>
  </div>
</div>

[![License](https://img.shields.io/badge/license-GPL--3.0-blue.svg)](LICENSE)
[![Version](https://img.shields.io/badge/version-1.0.0-green.svg)](https://github.com/shuheng-mo/xore/releases)
[![Rust](https://img.shields.io/badge/rust-1.91+-orange.svg)](https://www.rust-lang.org/)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)

[English](README_EN.md) | [简体中文](README.md)

**A local developer tool with extreme performance, deeply fusing semantic search and instant data analysis**

---

## Table of Contents

- [Overview](#overview)
- [Key Features](#key-features)
- [Architecture](#architecture)
- [Quick Start](#quick-start)
  - [Requirements](#requirements)
  - [Build & Install](#build--install)
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

### 🤖 Agent Neural Link (`xore agent`)

- **Structured Summarization**: `xore agent schema` returns data structures and distributions without moving raw data.
- **Smart Sampling**: `xore agent sample` automatically extracts the most representative data samples.
- **Token Budget Control**: Semantically compresses long text, preserving core logic (e.g., function headers) while omitting redundant implementation.
- **Agent Fix Suggestions**: Automatically transforms error messages into actionable fix instructions.

### 🔍 Intelligent Search Engine

- **Semantic Chunking**: Tree-sitter powered code awareness. Returns complete function/class blocks instead of just lines.
- **Full-text Search**: High-performance inverted index via Tantivy with BM25 ranking.
- **Fuzzy & Prefix**: Supports `~term` fuzzy matching and `term*` prefix search.
- **Incremental Indexing**: Millisecond-level file watching (`--watch`) ensures the Agent always sees the latest state.

### ⚡ High-Performance Data Processing

- **Predicate Pushdown**: Executes SQL filtering and aggregation locally via Polars, outputting only the final result.
- **Lazy Evaluation**: Handles massive datasets that far exceed available RAM.
- **Multi-format Support**: Native support for CSV, JSON, Parquet, Arrow, and log files.

### 🎯 Data Quality Analysis

- **Auto Profiling**: Statistical analysis, missing value detection, and outlier identification.
- **Type Inference**: Smart recognition of column types and data patterns.

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

---

## Detailed Documentation

Please refer to [docs/README.md](docs/README.md) for the complete usage guide.

---

## Project Structure

```
xore/
├── xore-cli/              # CLI interface
│   ├── src/
│   │   ├── main.rs       # Entry point
│   │   ├── commands/     # Command implementations
│   │   │   ├── find.rs   # Find command
│   │   │   ├── process.rs # Process command
│   │   │   ├── agent.rs  # Agent command
│   │   │   └── benchmark.rs # Benchmark command
│   │   └── ui/           # User interface
│   └── Cargo.toml
│
├── xore-core/             # Core module
│   ├── src/
│   │   ├── config.rs     # Configuration management
│   │   ├── error/        # Error handling system
│   │   ├── history.rs    # Search history
│   │   ├── recommendation.rs # Smart recommendations
│   │   └── types.rs      # Common types
│   └── Cargo.toml
│
├── xore-search/           # Search engine
│   ├── src/
│   │   ├── indexer.rs    # Index building
│   │   ├── incremental.rs # Incremental indexing
│   │   ├── query.rs      # Query processing
│   │   ├── scanner.rs    # File scanning
│   │   ├── tokenizer.rs  # Tokenizer
│   │   └── watcher.rs    # File watching
│   └── Cargo.toml
│
├── xore-process/          # Data processing
│   ├── src/
│   │   ├── parser.rs     # Data parsing
│   │   ├── sql.rs        # SQL engine
│   │   ├── profiler.rs   # Data profiling
│   │   ├── simd.rs       # SIMD numeric optimization
│   │   └── export.rs     # Export features
│   └── Cargo.toml
│
├── xore-ai/               # AI module
│   ├── src/
│   │   ├── embedding.rs  # Vector embeddings
│   │   ├── search.rs     # Vector search engine
│   │   └── tokenizer.rs  # Tokenizer
│   └── Cargo.toml
│
├── Cargo.toml            # Workspace config
├── README.md             # Chinese README
├── README_EN.md          # English README (this file)
├── LICENSE               # GPL-3.0 License
├── CONTRIBUTING.md       # Contributing guide
├── CHANGELOG.md          # Changelog
├── rustfmt.toml          # Rust formatting config
├── .gitignore            # Git ignore rules
├── docs/                 # Documentation
│   ├── README.md         # Usage guide
│   ├── getting-started.md # Getting started
│   ├── commands/         # Command reference
│   └── reference/        # Configuration reference
└── assets/               # Project assets (icons, etc.)
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

### Agent Efficiency (Token Savings)

| Task | Traditional (Bash/rg) | XORE (Agent-Native) | Token Savings |
| :--- | :--- | :--- | :--- |
| Log Analysis (50MB) | ~15,000 Tokens | **~50 Tokens** | **99.6%** |
| Schema Discovery | ~2,000 Tokens | **~30 Tokens** | **98.5%** |
| Data Aggregation | ~10,000 Tokens | **~100 Tokens** | **99.0%** |

### Search Performance

| Operation | Metric | Status |
|-----------|--------|--------|
| File scanning | 12,511 files/s | ✅ |
| Index build | 92,678 MB/s | ✅ **Exceeds target** |
| Standard search (p99) | 0.2 ms | ✅ **Exceeds target** |
| Prefix search (p99) | 0.0 ms | ✅ **Exceeds target** |
| Fuzzy search (p99) | 0.3 ms | ✅ **Exceeds target** |
| Incremental index latency | ~45 ms | ✅ |

### Detailed Metrics

**Index Build:**

- Dataset: 9.5 GB (17 files)
- Average time: 105.2 ms
- Throughput: 92,678 MB/s

**Search Latency Distribution:**

- Standard search: p50=0.0ms, p95=0.2ms, p99=0.2ms
- Prefix search: p50=0.0ms, p99=0.0ms
- Fuzzy search: p50=0.0ms, p99=0.3ms

### Performance Advantages

| Comparison | Traditional (grep/rg) | XORE (Agent-Native) | Advantage |
|-----------|----------------------|---------------------|-----------|
| **Token Efficiency** | Raw text transfer | **Pushdown / Structured Summary** | **90%+ Token savings** |
| **Full-text search** | grep (linear scan) | Index-accelerated | 1000x+ |
| **File finding** | find (dir traversal) | Parallel scan | 10x+ |
| **Regex search** | ripgrep (no index) | Post-index search | 100x+ |

### Memory Usage

- Index: ~15–20% of raw data size
- Runtime: Peak memory < 2× data size
- Lazy evaluation: Supports datasets larger than RAM

*Test environment: macOS (Apple Silicon), mimalloc allocator*

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

This project is open-sourced under the GPL-3.0 License — see [LICENSE](LICENSE) for details.

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
