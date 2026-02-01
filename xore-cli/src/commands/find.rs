//! Find 命令实现
//!
//! 提供文件扫描和搜索功能，集成了 FileScanner 进行高性能文件遍历，
//! 以及 IndexBuilder 和 Searcher 进行全文索引搜索。

use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{Context, Result};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use tracing::{debug, info};
use xore_search::{
    index_exists, FileScanner, FileTypeFilter, IndexBuilder, IndexConfig, MtimeFilter, ScanConfig,
    Searcher, SizeFilter,
};

/// Find 命令参数
pub struct FindArgs {
    pub query: Option<String>,
    pub path: String,
    pub file_type: Option<String>,
    pub size: Option<String>,
    pub mtime: Option<String>,
    pub max_depth: Option<usize>,
    pub hidden: bool,
    pub no_ignore: bool,
    pub follow_links: bool,
    pub threads: Option<usize>,
    pub semantic: bool,
    pub index: bool,
    pub rebuild: bool,
    pub index_dir: Option<String>,
}

/// 格式化文件大小为人类可读格式
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// 获取索引目录路径
fn get_index_path(args: &FindArgs) -> PathBuf {
    if let Some(ref dir) = args.index_dir {
        PathBuf::from(dir)
    } else {
        // 默认使用项目级索引
        let search_path = Path::new(&args.path);
        if search_path.is_absolute() {
            search_path.join(".xore/index")
        } else {
            PathBuf::from(".xore/index")
        }
    }
}

/// 执行查找命令
pub fn execute(args: FindArgs) -> Result<()> {
    info!("Starting find command with path: {}", args.path);

    // 如果启用了索引搜索模式
    if args.index {
        return execute_index_search(&args);
    }

    // 构建扫描配置
    let mut config = ScanConfig::new(&args.path);

    // 应用文件类型过滤
    if let Some(ref type_str) = args.file_type {
        let filter = FileTypeFilter::parse(type_str)
            .with_context(|| format!("Invalid file type filter: {}", type_str))?;
        config = config.with_file_type(filter);
        debug!("File type filter applied: {:?}", type_str);
    }

    // 应用文件大小过滤
    if let Some(ref size_str) = args.size {
        let filter = SizeFilter::parse(size_str)
            .with_context(|| format!("Invalid size filter: {}", size_str))?;
        config = config.with_size_filter(filter);
        debug!("Size filter applied: {:?}", size_str);
    }

    // 应用修改时间过滤
    if let Some(ref mtime_str) = args.mtime {
        let filter = MtimeFilter::parse(mtime_str)
            .with_context(|| format!("Invalid mtime filter: {}", mtime_str))?;
        config = config.with_mtime_filter(filter);
        debug!("Mtime filter applied: {:?}", mtime_str);
    }

    // 应用其他配置
    if let Some(depth) = args.max_depth {
        config = config.with_max_depth(depth);
    }

    config = config
        .with_include_hidden(args.hidden)
        .with_respect_gitignore(!args.no_ignore)
        .with_follow_links(args.follow_links);

    if let Some(threads) = args.threads {
        config = config.with_threads(threads);
    }

    // 显示扫描开始信息
    println!("{}", "扫描文件中...".cyan());

    // 创建进度条（仅在终端中显示）
    let spinner = if atty::is(atty::Stream::Stdout) {
        let pb = ProgressBar::new_spinner();
        pb.set_style(ProgressStyle::default_spinner().template("{spinner:.cyan} {msg}").unwrap());
        pb.set_message("正在扫描...");
        pb.enable_steady_tick(std::time::Duration::from_millis(100));
        Some(pb)
    } else {
        None
    };

    // 执行扫描
    let scanner = FileScanner::new(config);
    let (files, stats) = scanner.scan()?;

    // 关闭进度条
    if let Some(pb) = spinner {
        pb.finish_and_clear();
    }

    // 如果有查询字符串，进行内容搜索（目前只支持简单的文件名匹配）
    let matched_files: Vec<_> = if let Some(ref query) = args.query {
        if args.semantic {
            println!("{}", "语义搜索功能即将推出...".yellow());
            files
        } else {
            // 简单的文件名/路径匹配
            let query_lower = query.to_lowercase();
            files
                .into_iter()
                .filter(|f| f.path.to_string_lossy().to_lowercase().contains(&query_lower))
                .collect()
        }
    } else {
        files
    };

    // 按路径排序
    let mut sorted_files = matched_files;
    sorted_files.sort_by(|a, b| a.path.cmp(&b.path));

    // 显示结果
    println!();
    if sorted_files.is_empty() {
        println!("{}", "未找到匹配的文件".yellow());
    } else {
        for file in &sorted_files {
            let size_str = format_size(file.size);
            let path_str = file.path.display().to_string();

            // 高亮显示查询匹配的部分
            if let Some(ref query) = args.query {
                let highlighted = highlight_match(&path_str, query);
                println!("{:>10}  {}", size_str.dimmed(), highlighted);
            } else {
                println!("{:>10}  {}", size_str.dimmed(), path_str);
            }
        }
    }

    // 显示统计信息
    println!();
    println!(
        "{} 找到 {} 个文件 (共扫描 {} 个文件, {} 个目录, 耗时 {} ms)",
        "✓".green(),
        sorted_files.len().to_string().green().bold(),
        stats.total_files.to_string().cyan(),
        stats.directories.to_string().cyan(),
        stats.elapsed_ms.to_string().yellow()
    );

    if stats.total_size > 0 {
        println!("  总大小: {}", format_size(stats.total_size).cyan());
    }

    if stats.skipped > 0 {
        println!("  已跳过: {} 个文件 (不匹配过滤条件)", stats.skipped.to_string().dimmed());
    }

    if stats.errors > 0 {
        println!("  {} {} 个文件访问错误", "⚠".yellow(), stats.errors.to_string().yellow());
    }

    Ok(())
}

/// 执行全文索引搜索
fn execute_index_search(args: &FindArgs) -> Result<()> {
    let index_path = get_index_path(args);
    let start = Instant::now();

    // 检查是否需要构建/重建索引
    let need_build = args.rebuild || !index_exists(&index_path);

    if need_build {
        build_index(args, &index_path)?;
    }

    // 如果没有查询字符串，只构建索引
    let query = match &args.query {
        Some(q) => q,
        None => {
            println!("{}", "索引已准备就绪".green());
            return Ok(());
        }
    };

    // 执行搜索
    println!("{} 搜索 \"{}\"...", "🔍".cyan(), query);

    let searcher =
        Searcher::new(&index_path).with_context(|| format!("无法打开索引: {:?}", index_path))?;

    // 获取文件类型过滤
    let file_type_filter = args.file_type.as_deref();

    let results = if file_type_filter.is_some() {
        searcher.search_with_filter(query, file_type_filter, 100)?
    } else {
        searcher.search(query)?
    };

    let search_elapsed = start.elapsed();

    // 显示结果
    println!();
    if results.is_empty() {
        println!("{}", "未找到匹配结果".yellow());
    } else {
        for result in &results {
            let path_str = result.path.display().to_string();
            let score_str = format!("{:.2}", result.score);

            // 显示文件路径和行号
            let location = if let Some(line) = result.line {
                format!("{}:{}", path_str, line)
            } else {
                path_str
            };

            println!("{} {}", score_str.dimmed(), location.cyan());

            // 显示匹配片段
            if let Some(ref snippet) = result.snippet {
                // 片段已包含 ANSI 转义序列用于高亮
                let indented = snippet
                    .lines()
                    .map(|line| format!("    {}", line))
                    .collect::<Vec<_>>()
                    .join("\n");
                println!("{}", indented);
            }
            println!();
        }
    }

    // 显示统计信息
    println!(
        "{} 找到 {} 个匹配 (索引包含 {} 个文档, 耗时 {:.2?})",
        "✓".green(),
        results.len().to_string().green().bold(),
        searcher.num_docs().to_string().cyan(),
        search_elapsed
    );

    Ok(())
}

/// 构建索引
fn build_index(args: &FindArgs, index_path: &Path) -> Result<()> {
    let start = Instant::now();

    println!("{} 构建索引中...", "📑".cyan());

    // 创建进度条
    let spinner = if atty::is(atty::Stream::Stdout) {
        let pb = ProgressBar::new_spinner();
        pb.set_style(ProgressStyle::default_spinner().template("{spinner:.cyan} {msg}").unwrap());
        pb.set_message("扫描文件...");
        pb.enable_steady_tick(std::time::Duration::from_millis(100));
        Some(pb)
    } else {
        None
    };

    // 配置扫描器
    let mut scan_config = ScanConfig::new(&args.path);

    if let Some(ref type_str) = args.file_type {
        if let Ok(filter) = FileTypeFilter::parse(type_str) {
            scan_config = scan_config.with_file_type(filter);
        }
    }

    if let Some(depth) = args.max_depth {
        scan_config = scan_config.with_max_depth(depth);
    }

    scan_config = scan_config
        .with_include_hidden(args.hidden)
        .with_respect_gitignore(!args.no_ignore)
        .with_follow_links(args.follow_links);

    if let Some(threads) = args.threads {
        scan_config = scan_config.with_threads(threads);
    }

    // 执行扫描
    let scanner = FileScanner::new(scan_config);
    let (files, scan_stats) = scanner.scan()?;

    if let Some(ref pb) = spinner {
        pb.set_message(format!("索引 {} 个文件...", files.len()));
    }

    // 配置索引构建器
    let index_config = IndexConfig {
        index_path: index_path.to_path_buf(),
        writer_buffer_size: 50_000_000,   // 50MB
        max_file_size: 100 * 1024 * 1024, // 100MB
        use_mmap: true,
        mmap_threshold: 1024 * 1024, // 1MB
    };

    // 构建索引
    let mut builder = IndexBuilder::with_config(index_config)?;
    builder.add_documents_batch(&files)?;
    let index_stats = builder.build()?;

    // 关闭进度条
    if let Some(pb) = spinner {
        pb.finish_and_clear();
    }

    let elapsed = start.elapsed();

    // 显示构建统计
    println!(
        "{} 索引构建完成: {} 个文档 (扫描 {} 个文件, {} 个错误, 耗时 {:.2?})",
        "✓".green(),
        index_stats.documents_added.to_string().green().bold(),
        scan_stats.total_files.to_string().cyan(),
        index_stats.errors.len().to_string().yellow(),
        elapsed
    );

    if !index_stats.errors.is_empty() && index_stats.errors.len() <= 5 {
        println!("  错误:");
        for err in &index_stats.errors {
            println!("    {} {}", "•".red(), err.dimmed());
        }
    } else if index_stats.errors.len() > 5 {
        println!("  {} 超过 5 个错误，已省略详情", "⚠".yellow());
    }

    Ok(())
}

/// 高亮显示匹配的文本
fn highlight_match(text: &str, query: &str) -> String {
    let text_lower = text.to_lowercase();
    let query_lower = query.to_lowercase();

    if let Some(start) = text_lower.find(&query_lower) {
        let end = start + query.len();
        let before = &text[..start];
        let matched = &text[start..end];
        let after = &text[end..];

        format!("{}{}{}", before, matched.magenta().bold(), after)
    } else {
        text.to_string()
    }
}

// 添加 atty 检测终端的辅助模块
mod atty {
    pub enum Stream {
        Stdout,
    }

    pub fn is(_stream: Stream) -> bool {
        // 简单实现：检查是否为终端
        #[cfg(unix)]
        {
            use std::os::unix::io::AsRawFd;
            unsafe { libc::isatty(std::io::stdout().as_raw_fd()) != 0 }
        }
        #[cfg(windows)]
        {
            // Windows 简化实现
            true
        }
        #[cfg(not(any(unix, windows)))]
        {
            true
        }
    }
}
