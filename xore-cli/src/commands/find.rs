//! Find 命令实现
//!
//! 提供文件扫描和搜索功能，集成了 FileScanner 进行高性能文件遍历，
//! 以及 IndexBuilder 和 Searcher 进行全文索引搜索。

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{Context, Result};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use tracing::{debug, info, warn};
use xore_ai::{Document, EmbeddingModel, VectorSearcher};
use xore_config::XorePaths;
use xore_core::{
    format_time_ago, get_default_history_path, RecommendationEngine, SearchHistoryEntry, SearchType,
};
use xore_search::{
    index_exists, FileScanner, FileTypeFilter, IncrementalConfig, IncrementalIndexer, IndexBuilder,
    IndexConfig, MtimeFilter, ScanConfig, Searcher, SizeFilter, WatcherConfig,
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
    pub watch: bool,
    pub history: bool,
    pub recommend: bool,
    pub clear_history: bool,
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
///
/// 优先级：
/// 1. 用户通过 --index-dir 指定的路径
/// 2. 项目级索引（搜索目录下的 .xore/index）
/// 3. 全局索引 (~/.xore/index)
fn get_index_path(args: &FindArgs) -> PathBuf {
    // 1. 如果用户明确指定了索引目录，使用用户指定的路径
    if let Some(ref dir) = args.index_dir {
        return PathBuf::from(dir);
    }

    // 2. 检查是否存在项目级索引（搜索目录下的 .xore/index）
    let search_path = Path::new(&args.path);
    let project_index_path = if search_path.is_absolute() {
        search_path.join(".xore/index")
    } else {
        PathBuf::from(".xore/index")
    };

    if project_index_path.exists() || args.rebuild {
        return project_index_path;
    }

    // 3. 默认使用全局索引 (~/.xore/index)
    if let Ok(xore_paths) = XorePaths::new() {
        return xore_paths.index_dir();
    }

    // 备用方案：使用项目级索引
    project_index_path
}

/// 执行查找命令
pub fn execute(args: FindArgs) -> Result<()> {
    info!("Starting find command with path: {}", args.path);

    // 如果启用了watch模式，必须同时启用index模式
    if args.watch && !args.index {
        anyhow::bail!("--watch mode requires --index to be enabled");
    }

    // 处理历史记录相关命令
    if args.history {
        return show_search_history();
    }

    if args.recommend {
        return show_recommendations();
    }

    if args.clear_history {
        return clear_history();
    }

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

    // 如果有查询字符串，进行内容搜索
    let matched_files: Vec<_> = if let Some(ref query) = args.query {
        if args.semantic {
            // 执行语义搜索
            execute_semantic_search(&args, files)?
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

    // 记录搜索历史
    if let Some(ref query) = args.query {
        eprintln!("DEBUG: Recording search for query: {}", query);
        let search_type = if args.semantic { SearchType::Semantic } else { SearchType::FullText };

        match record_search_history(
            query,
            search_type,
            &args.path,
            sorted_files.len(),
            stats.elapsed_ms,
            args.file_type.clone(),
        ) {
            Ok(_) => {
                println!("  {} 已记录搜索历史", "✓".dimmed());
            }
            Err(e) => {
                eprintln!("DEBUG: Failed to record search history: {}", e);
            }
        }
    } else {
        eprintln!("DEBUG: No query to record");
    }

    Ok(())
}

/// 执行全文索引搜索
fn execute_index_search(args: &FindArgs) -> Result<()> {
    let index_path = get_index_path(args);

    // 如果启用了watch模式
    if args.watch {
        return execute_watch_mode(args, &index_path);
    }

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

    // 使用智能搜索，自动检测查询类型（前缀/模糊/标准）
    let results = if file_type_filter.is_some() {
        searcher.search_with_filter(query, file_type_filter, 100)?
    } else {
        // 使用智能搜索：支持 "term*" (前缀) 和 "~term" (模糊)
        searcher.search_smart(query, 100)?
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

    // 记录搜索历史
    let search_type =
        if args.file_type.is_some() { SearchType::FileType } else { SearchType::FullText };

    if let Err(e) = record_search_history(
        query,
        search_type,
        &args.path,
        results.len(),
        search_elapsed.as_millis() as u64,
        args.file_type.clone(),
    ) {
        debug!("Failed to record search history: {}", e);
    } else {
        println!("  {} 已记录搜索历史", "✓".dimmed());
    }

    Ok(())
}

/// 执行Watch模式（增量索引）
fn execute_watch_mode(args: &FindArgs, index_path: &Path) -> Result<()> {
    println!("{}", "启动增量索引监控模式...".cyan());

    // 检查是否需要先构建初始索引
    if args.rebuild || !index_exists(index_path) {
        build_index(args, index_path)?;
        println!();
    }

    // 创建增量索引配置
    let index_config = IndexConfig {
        index_path: index_path.to_path_buf(),
        writer_buffer_size: 50_000_000,
        max_file_size: 100 * 1024 * 1024,
        use_mmap: true,
        mmap_threshold: 1024 * 1024,
    };

    let watcher_config = WatcherConfig {
        debounce_duration: std::time::Duration::from_millis(500),
        batch_size: 50,
        exclude_patterns: vec![
            ".git".to_string(),
            "node_modules".to_string(),
            "target".to_string(),
            ".xore".to_string(),
            "*.tmp".to_string(),
            "*.swp".to_string(),
        ],
        include_hidden: args.hidden,
    };

    let incremental_config = IncrementalConfig {
        index_config,
        watcher_config,
        commit_threshold: 50,
        auto_commit_interval: 30,
    };

    // 创建运行时
    let rt = tokio::runtime::Runtime::new()?;

    rt.block_on(async {
        // 创建增量索引器
        let indexer = IncrementalIndexer::new(incremental_config)
            .await
            .context("Failed to create incremental indexer")?;

        // 开始监控
        let watch_path = PathBuf::from(&args.path);
        indexer.watch(&watch_path).await?;

        println!("{} 监控目录: {}", "🔍".cyan(), watch_path.display());
        println!("{} 按 Ctrl+C 停止监控", "💡".yellow());
        println!();

        // 启动统计报告任务
        let stats_indexer = std::sync::Arc::new(indexer);
        let stats_indexer_clone = stats_indexer.clone();
        let stats_task = tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                let stats = stats_indexer_clone.stats().await;
                if stats.created_count > 0 || stats.modified_count > 0 || stats.deleted_count > 0 {
                    println!(
                        "{} 统计: 创建 {}, 修改 {}, 删除 {}, 待提交 {}",
                        "📊".dimmed(),
                        stats.created_count.to_string().green(),
                        stats.modified_count.to_string().yellow(),
                        stats.deleted_count.to_string().red(),
                        stats.pending_changes.to_string().cyan()
                    );
                }
            }
        });

        // 使用 tokio::select! 同时运行事件循环和监听 Ctrl+C
        tokio::select! {
            result = stats_indexer.run() => {
                // 如果 run() 意外退出，报告错误
                result?;
            }
            _ = tokio::signal::ctrl_c() => {
                println!();
                println!("{}", "停止监控...".yellow());
            }
        }

        // 取消统计任务
        stats_task.abort();

        // 最后提交一次
        println!("{}", "提交最后的变更...".cyan());
        stats_indexer.commit().await?;

        Ok::<(), anyhow::Error>(())
    })?;

    println!("{}", "✓ 监控已停止".green());
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

/// 获取 ONNX 模型路径
fn get_model_path() -> PathBuf {
    // 优先使用环境变量
    if let Ok(path) = env::var("XORE_MODEL_PATH") {
        return PathBuf::from(path);
    }
    // 默认使用 ~/.xore/models/ 目录
    if let Ok(paths) = XorePaths::new() {
        return paths.models_dir().join("onnx/model.onnx");
    }
    // 回退到旧路径（用于开发）
    PathBuf::from("assets/models/onnx/model.onnx")
}

/// 获取 Tokenizer 路径
fn get_tokenizer_path() -> PathBuf {
    // 优先使用环境变量
    if let Ok(path) = env::var("XORE_TOKENIZER_PATH") {
        return PathBuf::from(path);
    }
    // 默认使用 ~/.xore/models/ 目录
    if let Ok(paths) = XorePaths::new() {
        return paths.models_dir().join("tokenizer.json");
    }
    // 回退到旧路径（用于开发）
    PathBuf::from("assets/models/tokenizer.json")
}

/// 读取文件内容（限制大小）
fn read_file_content(path: &Path, max_size: u64) -> Result<String> {
    let metadata = fs::metadata(path)?;

    // 跳过过大的文件
    if metadata.len() > max_size {
        anyhow::bail!("File too large: {} bytes", metadata.len());
    }

    // 跳过二进制文件（简单检测）
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    Ok(content)
}

/// 执行语义搜索
fn execute_semantic_search(
    args: &FindArgs,
    files: Vec<xore_search::ScannedFile>,
) -> Result<Vec<xore_search::ScannedFile>> {
    let query = match &args.query {
        Some(q) => q,
        None => {
            println!("{}", "语义搜索需要提供查询字符串".yellow());
            return Ok(files);
        }
    };

    println!("{}", "正在加载语义搜索模型...".cyan());

    // 加载模型
    let model_path = get_model_path();
    let tokenizer_path = get_tokenizer_path();

    if !model_path.exists() {
        anyhow::bail!(
            "模型文件不存在: {}\n提示: 请先下载模型，参考 docs/semantic-search-guide.md",
            model_path.display()
        );
    }

    if !tokenizer_path.exists() {
        anyhow::bail!(
            "Tokenizer 文件不存在: {}\n提示: 请先下载模型，参考 docs/semantic-search-guide.md",
            tokenizer_path.display()
        );
    }

    let model = EmbeddingModel::load(&model_path, &tokenizer_path)
        .context("Failed to load embedding model")?;

    println!("{}", "✓ 模型加载成功".green());

    // 创建向量搜索引擎
    let mut searcher = VectorSearcher::new(model);

    // 读取文件内容并建立索引
    println!("{}", "正在索引文件内容...".cyan());
    let max_file_size = 1024 * 1024; // 1MB
    let max_files = 1000; // 最多索引 1000 个文件

    let pb = ProgressBar::new(files.len().min(max_files) as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );

    let mut indexed_count = 0;
    let mut skipped_count = 0;

    for file in files.iter().take(max_files) {
        pb.inc(1);

        match read_file_content(&file.path, max_file_size) {
            Ok(content) => {
                if content.trim().is_empty() {
                    skipped_count += 1;
                    continue;
                }

                let doc = Document {
                    id: file.path.to_string_lossy().to_string(),
                    path: file.path.clone(),
                    content,
                };

                match searcher.add_document(doc) {
                    Ok(_) => {
                        indexed_count += 1;
                    }
                    Err(e) => {
                        warn!("Failed to index {}: {:#}", file.path.display(), e);
                        skipped_count += 1;
                    }
                }
            }
            Err(e) => {
                debug!("Skipped {}: {}", file.path.display(), e);
                skipped_count += 1;
            }
        }
    }

    pb.finish_and_clear();

    println!(
        "{} 已索引 {} 个文件 (跳过 {} 个)",
        "✓".green(),
        indexed_count.to_string().green().bold(),
        skipped_count.to_string().dimmed()
    );

    if indexed_count == 0 {
        println!("{}", "没有可索引的文件内容".yellow());
        return Ok(vec![]);
    }

    // 执行语义搜索
    println!("{}", format!("正在搜索: \"{}\"", query).cyan());
    let top_k = 20; // 返回前 20 个结果
    let results = searcher.search(query, top_k).context("Failed to perform semantic search")?;

    if results.is_empty() {
        println!("{}", "未找到相关结果".yellow());
        return Ok(vec![]);
    }

    // 将搜索结果转换回 FileInfo
    let matched_files: Vec<_> = results
        .into_iter()
        .filter_map(|result| {
            files.iter().find(|f| f.path == result.document.path).map(|f| {
                // 打印相似度分数
                println!(
                    "  {} (相似度: {:.4})",
                    f.path.display().to_string().cyan(),
                    result.score.to_string().yellow()
                );
                f.clone()
            })
        })
        .collect();

    Ok(matched_files)
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

/// 显示搜索历史
fn show_search_history() -> Result<()> {
    println!("{}", "📜 搜索历史".cyan());
    println!();

    let history_path = get_default_history_path();
    let engine = RecommendationEngine::new(Some(history_path))?;

    let recent = engine.get_recent_searches(10);

    if recent.is_empty() {
        println!("{}", "暂无搜索历史".yellow());
        return Ok(());
    }

    for (i, entry) in recent.iter().enumerate() {
        let time_ago = format_time_ago(&entry.timestamp);
        let type_str = entry.search_type.to_string();

        println!(
            "  {}. \"{}\" ({}) - {} - {} 结果 - {}",
            i + 1,
            entry.query.cyan(),
            type_str.dimmed(),
            entry.path.dimmed(),
            entry.result_count.to_string().green(),
            time_ago.dimmed()
        );
    }

    println!();
    println!("  总计: {} 条记录", engine.history_len().to_string().cyan());

    Ok(())
}

/// 显示智能推荐
fn show_recommendations() -> Result<()> {
    println!("{}", "💡 智能推荐".cyan());
    println!();

    let history_path = get_default_history_path();
    let engine = RecommendationEngine::new(Some(history_path))?;

    if engine.history_len() == 0 {
        println!("{}", "暂无足够的历史数据生成推荐".yellow());
        println!("{}", "请先进行一些搜索操作".dimmed());
        return Ok(());
    }

    // 生成推荐（使用空查询来获取通用推荐）
    let recommendations = engine.generate_recommendations("");

    if recommendations.is_empty() {
        println!("{}", "暂无推荐".yellow());
        return Ok(());
    }

    for (i, rec) in recommendations.iter().enumerate() {
        println!("  {}. {}", i + 1, rec.message.cyan());
        println!("     💡 {}", rec.suggestion.yellow());
        println!();
    }

    Ok(())
}

/// 清除搜索历史
fn clear_history() -> Result<()> {
    println!("{}", "🗑️ 清除搜索历史".cyan());
    println!();

    let history_path = get_default_history_path();
    let engine = RecommendationEngine::new(Some(history_path))?;

    let count = engine.clear_history()?;

    println!("{} 已清除 {} 条搜索记录", "✓".green(), count.to_string().green().bold());

    Ok(())
}

/// 记录搜索历史
fn record_search_history(
    query: &str,
    search_type: SearchType,
    path: &str,
    result_count: usize,
    execution_time_ms: u64,
    file_type: Option<String>,
) -> Result<()> {
    let history_path = get_default_history_path();
    info!("Creating recommendation engine at: {:?}", history_path);
    let engine = RecommendationEngine::new(Some(history_path))?;

    let entry = SearchHistoryEntry::new(
        query.to_string(),
        search_type,
        path.to_string(),
        result_count,
        execution_time_ms,
        file_type,
    );

    info!("Calling record_search with entry: {:?}", entry);
    engine.record_search(entry)?;
    info!("Search history recorded successfully");
    Ok(())
}
