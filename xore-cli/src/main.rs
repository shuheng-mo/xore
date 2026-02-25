//! XORE CLI - 命令行入口
// 全局内存分配器配置 - 使用 mimalloc 提升性能
#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use clap::{Parser, Subcommand};
use xore_core::LogConfig;

mod commands;
mod ui;

use commands::{benchmark, find, process};

/// XORE - 搜索和数据处理一体化工具
#[derive(Parser)]
#[command(name = "xore")]
#[command(author = "XORE Team")]
#[command(version = "1.0.0")]
#[command(about = "Explore the Abyss, Extract the Core", long_about = None)]
struct Cli {
    /// 详细输出模式
    #[arg(short, long)]
    verbose: bool,

    /// 静默模式
    #[arg(short, long)]
    quiet: bool,

    /// 禁用彩色输出
    #[arg(long)]
    no_color: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 查找文件（搜索功能）
    #[command(alias = "f")]
    Find {
        /// 查询字符串（可选，如不提供则只扫描文件）
        query: Option<String>,

        /// 搜索路径
        #[arg(long, default_value = ".")]
        path: String,

        /// 文件类型过滤（csv, json, log, code, text, parquet, 或逗号分隔的扩展名）
        #[arg(long, short = 't')]
        r#type: Option<String>,

        /// 文件大小过滤（例如：">1MB", "<500KB", "1MB-10MB"）
        #[arg(long, short = 's', allow_hyphen_values = true)]
        size: Option<String>,

        /// 修改时间过滤（例如："-7d" 过去7天, "+30d" 超过30天, "2024-01-01"）
        #[arg(long, short = 'm', allow_hyphen_values = true)]
        mtime: Option<String>,

        /// 最大遍历深度
        #[arg(long, short = 'd')]
        max_depth: Option<usize>,

        /// 包含隐藏文件
        #[arg(long)]
        hidden: bool,

        /// 不遵守 .gitignore 规则
        #[arg(long)]
        no_ignore: bool,

        /// 跟随符号链接
        #[arg(long, short = 'L')]
        follow_links: bool,

        /// 并行线程数（默认自动检测）
        #[arg(long, short = 'j')]
        threads: Option<usize>,

        /// 启用语义搜索
        #[arg(long)]
        semantic: bool,

        /// 启用全文索引搜索模式
        #[arg(long, short = 'i')]
        index: bool,

        /// 强制重建索引
        #[arg(long)]
        rebuild: bool,

        /// 指定索引目录路径
        #[arg(long)]
        index_dir: Option<String>,

        /// 启用文件监控模式（增量索引）
        #[arg(long, short = 'w')]
        watch: bool,
    },

    /// 处理数据
    #[command(alias = "p")]
    Process {
        /// 文件路径
        file: String,

        /// SQL查询
        query: Option<String>,

        /// 数据质量检测
        #[arg(long)]
        quality_check: bool,
    },

    /// 性能基准测试
    #[command(alias = "bench")]
    Benchmark {
        /// 测试套件 (all, scan, search, process, io, alloc)
        #[arg(long, short = 's', default_value = "all")]
        suite: benchmark::BenchmarkSuite,

        /// 输出格式 (text, json, csv)
        #[arg(long, short = 'o', default_value = "text")]
        output: benchmark::OutputFormat,

        /// 迭代次数
        #[arg(long, short = 'n', default_value = "3")]
        iterations: usize,

        /// 测试数据路径
        #[arg(long)]
        data_path: Option<String>,

        /// 预热次数
        #[arg(long, default_value = "1")]
        warmup: usize,
    },
}

fn main() -> anyhow::Result<()> {
    // 解析命令行参数
    let cli = Cli::parse();

    // 初始化日志系统
    let log_config = LogConfig::from_args(cli.verbose, cli.quiet, cli.no_color);
    log_config.init()?;

    // 执行子命令
    match cli.command {
        Commands::Find {
            query,
            path,
            r#type,
            size,
            mtime,
            max_depth,
            hidden,
            no_ignore,
            follow_links,
            threads,
            semantic,
            index,
            rebuild,
            index_dir,
            watch,
        } => {
            find::execute(find::FindArgs {
                query,
                path,
                file_type: r#type,
                size,
                mtime,
                max_depth,
                hidden,
                no_ignore,
                follow_links,
                threads,
                semantic,
                index,
                rebuild,
                index_dir,
                watch,
            })?;
        }
        Commands::Process { file, query, quality_check } => {
            process::execute(&file, query.as_deref(), quality_check)?;
        }
        Commands::Benchmark { suite, output, iterations, data_path, warmup } => {
            benchmark::execute(benchmark::BenchmarkArgs {
                suite,
                output,
                iterations,
                data_path,
                warmup,
            })?;
        }
    }

    Ok(())
}
