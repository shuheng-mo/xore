//! XORE CLI - 命令行入口
// 全局内存分配器配置 - 使用 mimalloc 提升性能
#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use clap::{Parser, Subcommand};
use xore_core::{print_anyhow_error, LogConfig};

mod commands;
mod ui;

use commands::{abyss, agent, benchmark, config, find, peek, process, watch};

/// XORE - 搜索和数据处理一体化工具
#[derive(Parser)]
#[command(name = "xore")]
#[command(author = "XORE Team")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Explore the Abyss, Extract the Core", long_about = None)]
struct Cli {
    /// 详细输出模式
    #[arg(short, long, global = true)]
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
    /// Agent 优化功能（Agent-Native）
    #[command(alias = "a")]
    Agent {
        #[command(subcommand)]
        subcommand: AgentCommands,
    },

    /// 文件监控守护进程管理
    Watch {
        #[command(subcommand)]
        subcommand: WatchCommands,
    },

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

        /// 以后台守护进程模式启动文件监控（P2.2）
        #[arg(long)]
        watch_daemon: bool,

        /// 守护进程监控的包含模式（逗号分隔扩展名，例如 "*.rs,*.toml"）
        #[arg(long)]
        watch_include: Option<String>,

        /// 守护进程监控的排除模式（逗号分隔目录，例如 "target,node_modules"）
        #[arg(long)]
        watch_exclude: Option<String>,

        /// 显示搜索历史
        #[arg(long)]
        history: bool,

        /// 显示智能推荐
        #[arg(long)]
        recommend: bool,

        /// 清除搜索历史
        #[arg(long)]
        clear_history: bool,

        /// 输出格式: raw (默认), json, agent-json, agent-md
        #[arg(long, short = 'o')]
        output: Option<String>,

        /// 限制输出 token 数量
        #[arg(long)]
        max_tokens: Option<usize>,
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

        /// 输出文件路径（支持 csv, json, parquet 格式）
        #[arg(long, short = 'o')]
        output: Option<String>,

        /// 导出格式（如果不指定，从输出文件扩展名推断）
        #[arg(long, short = 'f')]
        format: Option<String>,
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

    /// 管理全局配置
    Config {
        #[command(subcommand)]
        subcommand: ConfigCommands,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// 显示当前配置
    Show,

    /// 获取配置项的值
    Get {
        /// 配置项名称（例如：paths.index, search.max_file_size_mb）
        key: String,
    },

    /// 设置配置项的值
    Set {
        /// 配置项名称
        key: String,
        /// 配置项新值
        value: String,
    },

    /// 重置配置为默认值
    Reset,

    /// 编辑配置文件
    Edit,
}

#[derive(Subcommand)]
enum AgentCommands {
    /// 生成 Agent 初始化 Prompt
    Init {
        /// 目标模型 (gpt-4, claude, ollama, deepseek)
        #[arg(long, default_value = "gpt-4")]
        model: String,

        /// 输出格式: mcp (默认), openai, langchain, openapi
        #[arg(long, default_value = "mcp")]
        format: String,
    },

    /// 获取数据结构（不读全文）
    Schema {
        /// 文件路径
        file: String,

        /// 显示分布直方图
        #[arg(long)]
        histogram: bool,

        /// JSON 格式输出
        #[arg(long)]
        json: bool,

        /// 压缩 JSON（无空格）
        #[arg(long)]
        minify: bool,

        /// 注入当前会话上下文到输出
        #[arg(long)]
        with_context: bool,
    },

    /// 智能采样数据
    Sample {
        /// 文件路径
        file: String,

        /// 采样行数
        #[arg(short = 'n', long, default_value = "5")]
        rows: usize,

        /// 采样策略 (random, head, tail, smart)
        #[arg(long, default_value = "smart")]
        strategy: String,

        /// JSON 格式输出
        #[arg(long)]
        json: bool,

        /// 注入当前会话上下文到输出
        #[arg(long)]
        with_context: bool,
    },

    /// 执行 SQL 查询
    Query {
        /// 文件路径
        file: String,

        /// SQL 查询语句
        sql: String,

        /// 输出格式 (json, csv, table)
        #[arg(long, short = 'f', default_value = "json")]
        format: String,

        /// 压缩 JSON（无空格）
        #[arg(long)]
        minify: bool,

        /// 返回行数限制
        #[arg(long)]
        limit: Option<usize>,

        /// 注入当前会话上下文到输出
        #[arg(long)]
        with_context: bool,
    },

    /// SQL 错误分析
    Explain {
        /// SQL 语句
        sql: String,
    },

    /// 会话上下文管理
    Context {
        #[command(subcommand)]
        subcommand: ContextCommands,
    },

    /// 目录扫描与智能预览（P2.1）
    Peek {
        /// 目标目录
        directory: String,

        /// 读取指定文件
        #[arg(long)]
        file: Option<String>,

        /// 禁用缓存（强制重新扫描）
        #[arg(long)]
        no_cache: bool,

        /// 输出格式：json（默认）| tree | md
        #[arg(long, short = 'o', default_value = "json")]
        output: String,

        /// 最大扫描深度（默认 5）
        #[arg(long, short = 'd')]
        max_depth: Option<usize>,

        /// 包含模式（逗号分隔的扩展名，例如 "*.rs,*.toml"）
        #[arg(long)]
        include: Option<String>,

        /// 排除模式（逗号分隔，例如 "target,node_modules"）
        #[arg(long)]
        exclude: Option<String>,
    },

    /// 全局文件监控守护进程（P2.3）
    Abyss {
        /// 启动全局监控
        #[arg(long)]
        start: bool,

        /// 强制启动（跳过确认）
        #[arg(long)]
        force: bool,

        /// 查看监控状态
        #[arg(long)]
        status: bool,

        /// 查看监控日志
        #[arg(long)]
        logs: bool,

        /// 日志显示行数（配合 --logs 使用）
        #[arg(long, default_value = "50")]
        lines: usize,

        /// 停止全局监控
        #[arg(long)]
        stop: bool,

        /// 显示或修改配置
        #[arg(long)]
        config: bool,

        /// 排除的目录（逗号分隔，例如 "Downloads,Desktop"）
        #[arg(long)]
        exclude: Option<String>,

        /// 仅监控的扩展名（逗号分隔，例如 "rs,toml"）
        #[arg(long)]
        include: Option<String>,
    },
}

/// Watch 命令（顶级）
#[derive(Subcommand)]
enum WatchCommands {
    /// 查看所有监控守护进程状态
    Status,

    /// 查看监控日志
    Logs {
        /// 显示行数
        #[arg(long, short = 'n', default_value = "100")]
        lines: usize,
    },

    /// 停止守护进程
    Stop {
        /// 指定要停止的监控路径（不指定则停止所有）
        #[arg(long)]
        path: Option<String>,
    },
}

#[derive(Subcommand)]
enum ContextCommands {
    /// 获取会话摘要
    Get {
        /// 摘要级别: short | detailed
        #[arg(long, default_value = "short")]
        level: String,

        /// 会话 ID（默认 "default"）
        #[arg(long, default_value = "default")]
        session_id: String,
    },

    /// 清空会话记录
    Clear {
        /// 会话 ID（默认 "default"）
        #[arg(long, default_value = "default")]
        session_id: String,
    },

    /// 导出会话为 JSON
    Export {
        /// 会话 ID（默认 "default"）
        #[arg(long, default_value = "default")]
        session_id: String,
    },

    /// 设置自定义上下文
    Set {
        /// 自定义上下文文本
        #[arg(long)]
        custom: String,

        /// 会话 ID（默认 "default"）
        #[arg(long, default_value = "default")]
        session_id: String,
    },
}

/// 执行命令的内部函数
fn run_command(cli: &Cli) -> anyhow::Result<()> {
    match &cli.command {
        Commands::Agent { subcommand } => {
            let agent_args = match subcommand {
                AgentCommands::Init { model, format } => agent::AgentArgs {
                    subcommand: agent::AgentSubcommand::Init {
                        model: model.clone(),
                        format: format.clone(),
                    },
                },
                AgentCommands::Schema { file, histogram, json, minify, with_context } => {
                    agent::AgentArgs {
                        subcommand: agent::AgentSubcommand::Schema {
                            file: file.clone(),
                            histogram: *histogram,
                            json: *json,
                            minify: *minify,
                            with_context: *with_context,
                        },
                    }
                }
                AgentCommands::Sample { file, rows, strategy, json, with_context } => {
                    let strategy = strategy.parse().unwrap_or(agent::SampleStrategy::Smart);
                    agent::AgentArgs {
                        subcommand: agent::AgentSubcommand::Sample {
                            file: file.clone(),
                            n: *rows,
                            strategy,
                            json: *json,
                            with_context: *with_context,
                        },
                    }
                }
                AgentCommands::Query { file, sql, format, minify, limit, with_context } => {
                    agent::AgentArgs {
                        subcommand: agent::AgentSubcommand::Query {
                            file: file.clone(),
                            sql: sql.clone(),
                            format: format.clone(),
                            minify: *minify,
                            limit: *limit,
                            with_context: *with_context,
                        },
                    }
                }
                AgentCommands::Explain { sql } => agent::AgentArgs {
                    subcommand: agent::AgentSubcommand::Explain { sql: sql.clone() },
                },
                AgentCommands::Context { subcommand } => {
                    let ctx_sub = match subcommand {
                        ContextCommands::Get { level, session_id } => {
                            agent::ContextSubcommand::Get {
                                level: level.clone(),
                                session_id: session_id.clone(),
                            }
                        }
                        ContextCommands::Clear { session_id } => {
                            agent::ContextSubcommand::Clear { session_id: session_id.clone() }
                        }
                        ContextCommands::Export { session_id } => {
                            agent::ContextSubcommand::Export { session_id: session_id.clone() }
                        }
                        ContextCommands::Set { custom, session_id } => {
                            agent::ContextSubcommand::Set {
                                custom: custom.clone(),
                                session_id: session_id.clone(),
                            }
                        }
                    };
                    agent::AgentArgs {
                        subcommand: agent::AgentSubcommand::Context { subcommand: ctx_sub },
                    }
                }
                // P2.1: peek 命令
                AgentCommands::Peek {
                    directory,
                    file,
                    no_cache,
                    output,
                    max_depth,
                    include,
                    exclude,
                } => {
                    peek::execute(peek::PeekArgs {
                        directory: directory.clone(),
                        file: file.clone(),
                        use_cache: !no_cache,
                        output: output.clone(),
                        max_depth: *max_depth,
                        include: include.clone(),
                        exclude: exclude.clone(),
                    })?;
                    return Ok(());
                }
                // P2.3: abyss 命令
                AgentCommands::Abyss {
                    start,
                    force,
                    status,
                    logs,
                    lines,
                    stop,
                    config,
                    exclude,
                    include,
                } => {
                    let action = if *start {
                        abyss::AbyssAction::Start {
                            force: *force,
                            exclude: exclude.clone(),
                            include: include.clone(),
                        }
                    } else if *status {
                        abyss::AbyssAction::Status
                    } else if *logs {
                        abyss::AbyssAction::Logs { lines: *lines }
                    } else if *stop {
                        abyss::AbyssAction::Stop
                    } else if *config {
                        abyss::AbyssAction::Config {
                            exclude: exclude.clone(),
                            include: include.clone(),
                        }
                    } else {
                        // 默认显示状态
                        abyss::AbyssAction::Status
                    };
                    abyss::execute(abyss::AbyssArgs { action })?;
                    return Ok(());
                }
            };
            agent::execute(agent_args)?;
        }
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
            watch_daemon,
            watch_include,
            watch_exclude,
            history,
            recommend,
            clear_history,
            output,
            max_tokens,
        } => {
            find::execute(find::FindArgs {
                query: query.clone(),
                path: path.clone(),
                file_type: r#type.clone(),
                size: size.clone(),
                mtime: mtime.clone(),
                max_depth: *max_depth,
                hidden: *hidden,
                no_ignore: *no_ignore,
                follow_links: *follow_links,
                threads: *threads,
                semantic: *semantic,
                index: *index,
                rebuild: *rebuild,
                index_dir: index_dir.clone(),
                watch: *watch,
                watch_daemon: *watch_daemon,
                watch_include: watch_include.clone(),
                watch_exclude: watch_exclude.clone(),
                history: *history,
                recommend: *recommend,
                clear_history: *clear_history,
                output: output.clone(),
                max_tokens: *max_tokens,
            })?;
        }
        // P2.2: Watch 守护进程管理
        Commands::Watch { subcommand } => {
            let watch_args = match subcommand {
                WatchCommands::Status => {
                    watch::WatchArgs { subcommand: watch::WatchSubcommand::Status }
                }
                WatchCommands::Logs { lines } => {
                    watch::WatchArgs { subcommand: watch::WatchSubcommand::Logs { lines: *lines } }
                }
                WatchCommands::Stop { path } => watch::WatchArgs {
                    subcommand: watch::WatchSubcommand::Stop { path: path.clone() },
                },
            };
            watch::execute(watch_args)?;
        }
        Commands::Process { file, query, quality_check, output, format } => {
            process::execute(
                file,
                query.as_deref(),
                *quality_check,
                output.as_deref(),
                format.as_deref(),
            )?;
        }
        Commands::Benchmark { suite, output, iterations, data_path, warmup } => {
            benchmark::execute(benchmark::BenchmarkArgs {
                suite: *suite,
                output: *output,
                iterations: *iterations,
                data_path: data_path.clone(),
                warmup: *warmup,
            })?;
        }
        Commands::Config { subcommand } => {
            let config_args = match subcommand {
                ConfigCommands::Show => {
                    config::ConfigArgs { subcommand: config::ConfigSubcommand::Show }
                }
                ConfigCommands::Get { key } => config::ConfigArgs {
                    subcommand: config::ConfigSubcommand::Get { key: key.clone() },
                },
                ConfigCommands::Set { key, value } => config::ConfigArgs {
                    subcommand: config::ConfigSubcommand::Set {
                        key: key.clone(),
                        value: value.clone(),
                    },
                },
                ConfigCommands::Reset => {
                    config::ConfigArgs { subcommand: config::ConfigSubcommand::Reset }
                }
                ConfigCommands::Edit => {
                    config::ConfigArgs { subcommand: config::ConfigSubcommand::Edit }
                }
            };
            config::execute(config_args)?;
        }
    }
    Ok(())
}

fn main() {
    // 解析命令行参数
    let cli = Cli::parse();

    // 初始化日志系统
    let log_config = LogConfig::from_args(cli.verbose, cli.quiet, cli.no_color);
    if let Err(e) = log_config.init() {
        print_anyhow_error(&e, cli.verbose, cli.no_color);
        std::process::exit(1);
    }

    // 执行子命令
    if let Err(e) = run_command(&cli) {
        // BrokenPipe 是正常的管道截断（如 | head），静默退出
        if let Some(io_err) = e.downcast_ref::<std::io::Error>() {
            if io_err.kind() == std::io::ErrorKind::BrokenPipe {
                std::process::exit(0);
            }
        }
        print_anyhow_error(&e, cli.verbose, cli.no_color);
        std::process::exit(1);
    }
}
