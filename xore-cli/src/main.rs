//! XORE CLI - 命令行入口

use clap::{Parser, Subcommand};
use xore_core::LogConfig;

mod commands;
mod ui;

use commands::{find, process};

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
        /// 查询字符串
        query: String,

        /// 搜索路径
        #[arg(long, default_value = ".")]
        path: String,

        /// 文件类型
        #[arg(long)]
        r#type: Option<String>,

        /// 启用语义搜索
        #[arg(long)]
        semantic: bool,
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
}

fn main() -> anyhow::Result<()> {
    // 解析命令行参数
    let cli = Cli::parse();

    // 初始化日志系统
    let log_config = LogConfig::from_args(cli.verbose, cli.quiet, cli.no_color);
    log_config.init()?;

    // 执行子命令
    match cli.command {
        Commands::Find { query, path, r#type, semantic } => {
            find::execute(&query, &path, r#type.as_deref(), semantic)?;
        }
        Commands::Process { file, query, quality_check } => {
            process::execute(&file, query.as_deref(), quality_check)?;
        }
    }

    Ok(())
}
