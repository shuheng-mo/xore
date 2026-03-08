//! Peek 命令实现 - 让智能体"偷看"文件内容
//!
//! 扫描目录，返回结构化摘要，替代 `ls`、`cat` 等命令，优化工作流程。
//!
//! ## 使用示例
//!
//! ```bash
//! xore agent peek src/         # 扫描目录，返回结构化摘要
//! xore agent peek src/ --file main.rs  # 读取指定文件内容
//! xore agent peek src/ --no-cache      # 强制刷新缓存
//! xore agent peek src/ --output tree   # 树形输出
//! xore agent peek src/ --output md     # Markdown 表格输出
//! ```

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::Result;
use colored::*;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

/// 文件条目（给 LLM 看的精简信息）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    /// 相对路径
    pub path: String,
    /// 文件大小（字节）
    pub size: u64,
    /// 行数（仅文本文件）
    pub lines: Option<usize>,
    /// 文件类型：code | config | text | binary
    pub file_type: String,
    /// 极短摘要（给 LLM 看）
    pub summary: String,
    /// 最后修改时间（ISO 8601）
    pub modified: Option<String>,
}

/// 目录摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirSummary {
    /// 根目录
    pub root: String,
    /// 文件列表
    pub files: Vec<FileEntry>,
    /// 总文件数
    pub total_files: usize,
    /// 总大小（字节）
    pub total_size: u64,
    /// 扫描耗时（毫秒）
    pub elapsed_ms: u64,
    /// 缓存信息
    pub cache_info: Option<CacheInfo>,
}

/// 缓存信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheInfo {
    pub cached: bool,
    pub cache_key: String,
    pub expires_at: Option<String>,
}

/// 内存缓存（目录路径 -> (摘要, 缓存时间)）
static PEEK_CACHE: Lazy<Mutex<HashMap<String, (DirSummary, Instant)>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Peek 命令参数
pub struct PeekArgs {
    /// 目标目录
    pub directory: String,
    /// 读取指定文件
    pub file: Option<String>,
    /// 是否使用缓存（默认开启）
    pub use_cache: bool,
    /// 输出格式：json | tree | md
    pub output: String,
    /// 最大扫描深度
    pub max_depth: Option<usize>,
    /// 包含模式（逗号分隔的扩展名，例如 "*.rs,*.toml"）
    pub include: Option<String>,
    /// 排除模式（逗号分隔的目录/文件，例如 "target,node_modules"）
    pub exclude: Option<String>,
}

impl Default for PeekArgs {
    fn default() -> Self {
        Self {
            directory: ".".to_string(),
            file: None,
            use_cache: true,
            output: "json".to_string(),
            max_depth: None,
            include: None,
            exclude: None,
        }
    }
}

/// 执行 peek 命令
pub fn execute(args: PeekArgs) -> Result<()> {
    let dir_path = Path::new(&args.directory);

    if !dir_path.exists() {
        return Err(anyhow::anyhow!("目录不存在: {}", args.directory));
    }

    // 如果指定了 --file，读取单个文件
    if let Some(ref file_name) = args.file {
        return peek_single_file(dir_path, file_name, &args.output);
    }

    // 生成缓存 key（路径 + 目录最后修改时间）
    let cache_key = make_cache_key(dir_path);
    let cache_ttl = Duration::from_secs(300); // 5 分钟

    // 检查缓存
    if args.use_cache {
        let cache = PEEK_CACHE.lock().unwrap();
        if let Some((summary, timestamp)) = cache.get(&cache_key) {
            if timestamp.elapsed() < cache_ttl {
                let mut cached = summary.clone();
                // 更新缓存信息
                cached.cache_info = Some(CacheInfo {
                    cached: true,
                    cache_key: cache_key.clone(),
                    expires_at: Some(format_expires_at(timestamp, cache_ttl)),
                });
                return print_summary(&cached, &args.output);
            }
        }
    }

    // 扫描目录
    let start = Instant::now();
    let summary = scan_directory(dir_path, &args, &cache_key, start)?;

    // 存入缓存
    if args.use_cache {
        let mut cache = PEEK_CACHE.lock().unwrap();
        cache.insert(cache_key.clone(), (summary.clone(), Instant::now()));
    }

    print_summary(&summary, &args.output)
}

/// 读取单个文件内容
fn peek_single_file(dir: &Path, file_name: &str, output: &str) -> Result<()> {
    // 先在目录中查找文件
    let file_path = if Path::new(file_name).is_absolute() {
        PathBuf::from(file_name)
    } else {
        dir.join(file_name)
    };

    if !file_path.exists() {
        // 递归搜索
        let found = find_file_recursive(dir, file_name)?;
        if found.is_empty() {
            return Err(anyhow::anyhow!("文件未找到: {} (在 {} 中搜索)", file_name, dir.display()));
        }
        if found.len() > 1 {
            eprintln!("{} 找到多个匹配文件，使用第一个:", "⚠️".yellow());
            for f in &found {
                eprintln!("  - {}", f.display());
            }
        }
        return read_and_print_file(&found[0], output);
    }

    read_and_print_file(&file_path, output)
}

/// 递归搜索文件
fn find_file_recursive(dir: &Path, file_name: &str) -> Result<Vec<PathBuf>> {
    let mut results = Vec::new();
    find_file_in_dir(dir, file_name, &mut results, 0, 5)?;
    Ok(results)
}

fn find_file_in_dir(
    dir: &Path,
    file_name: &str,
    results: &mut Vec<PathBuf>,
    depth: usize,
    max_depth: usize,
) -> Result<()> {
    if depth > max_depth {
        return Ok(());
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        // 跳过隐藏目录和特殊目录
        if name.starts_with('.') || matches!(name, "target" | "node_modules" | "__pycache__") {
            continue;
        }

        if path.is_dir() {
            find_file_in_dir(&path, file_name, results, depth + 1, max_depth)?;
        } else if name == file_name {
            results.push(path);
        }
    }

    Ok(())
}

/// 读取并打印文件内容
fn read_and_print_file(path: &Path, output: &str) -> Result<()> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let (file_type, _) = detect_file_type(ext);

    if file_type == "binary" {
        match output {
            "json" => {
                let result = serde_json::json!({
                    "path": path.display().to_string(),
                    "file_type": "binary",
                    "content": null,
                    "note": "binary file, cannot display"
                });
                println!("{}", serde_json::to_string_pretty(&result)?);
            }
            _ => {
                println!("{} [binary file]", path.display());
            }
        }
        return Ok(());
    }

    let content = fs::read_to_string(path)?;
    let lines: Vec<&str> = content.lines().collect();
    let preview = get_preview_from_lines(&lines, 15, 10);
    let compressed = compress_code(&preview, ext);

    match output {
        "json" => {
            let result = serde_json::json!({
                "path": path.display().to_string(),
                "file_type": file_type,
                "total_lines": lines.len(),
                "content": compressed,
                "truncated": lines.len() > 25,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        _ => {
            println!("{} {} ({} 行):", "📄".cyan(), path.display().to_string().cyan(), lines.len());
            println!("{}", compressed);
        }
    }

    Ok(())
}

/// 扫描目录
fn scan_directory(
    dir: &Path,
    args: &PeekArgs,
    cache_key: &str,
    start: Instant,
) -> Result<DirSummary> {
    // 解析忽略规则
    let mut default_ignores =
        vec![".git", "node_modules", "target", "__pycache__", "venv", ".xore"];

    let extra_excludes: Vec<String> = args
        .exclude
        .as_deref()
        .unwrap_or("")
        .split(',')
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect();

    for exc in &extra_excludes {
        default_ignores.push(exc.as_str());
    }

    // 解析包含模式
    let include_exts: Vec<String> = args
        .include
        .as_deref()
        .unwrap_or("")
        .split(',')
        .filter(|s| !s.is_empty())
        .map(|s| s.trim_start_matches("*.").trim_start_matches('.').to_lowercase())
        .collect();

    let max_depth = args.max_depth.unwrap_or(5);

    // 递归扫描
    let mut files = Vec::new();
    let mut total_size = 0u64;
    scan_dir_recursive(
        dir,
        dir,
        &default_ignores,
        &include_exts,
        max_depth,
        0,
        &mut files,
        &mut total_size,
    )?;

    // 排序：优先显示目录根文件
    files.sort_by(|a, b| a.path.cmp(&b.path));

    let elapsed_ms = start.elapsed().as_millis() as u64;
    let total_files = files.len();

    Ok(DirSummary {
        root: dir.display().to_string(),
        files,
        total_files,
        total_size,
        elapsed_ms,
        cache_info: Some(CacheInfo {
            cached: false,
            cache_key: cache_key.to_string(),
            expires_at: None,
        }),
    })
}

/// 递归扫描目录
#[allow(clippy::too_many_arguments)]
fn scan_dir_recursive(
    root: &Path,
    dir: &Path,
    ignores: &[&str],
    include_exts: &[String],
    max_depth: usize,
    current_depth: usize,
    files: &mut Vec<FileEntry>,
    total_size: &mut u64,
) -> Result<()> {
    if current_depth > max_depth {
        return Ok(());
    }

    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return Ok(()), // 跳过无权访问的目录
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        // 跳过隐藏文件
        if name.starts_with('.') {
            continue;
        }

        // 检查忽略列表
        if ignores.iter().any(|ig| name == *ig || path.to_string_lossy().contains(ig)) {
            continue;
        }

        if path.is_dir() {
            // 递归扫描子目录
            scan_dir_recursive(
                root,
                &path,
                ignores,
                include_exts,
                max_depth,
                current_depth + 1,
                files,
                total_size,
            )?;
        } else if path.is_file() {
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();

            // 应用包含过滤
            if !include_exts.is_empty() && !include_exts.contains(&ext) {
                continue;
            }

            let metadata = match fs::metadata(&path) {
                Ok(m) => m,
                Err(_) => continue,
            };

            let size = metadata.len();
            *total_size += size;

            // 获取相对路径
            let rel_path = path.strip_prefix(root).unwrap_or(&path);
            let rel_path_str = rel_path.display().to_string();

            // 获取修改时间
            let modified = metadata.modified().ok().map(format_system_time);

            // 判断文件类型和生成摘要
            let (file_type, summary) = get_file_summary(&path, &ext, size);

            // 统计行数（仅对文本文件，且文件不超过 1MB）
            let lines =
                if file_type != "binary" && size < 1024 * 1024 { count_lines(&path) } else { None };

            files.push(FileEntry { path: rel_path_str, size, lines, file_type, summary, modified });
        }
    }

    Ok(())
}

/// 检测文件类型
fn detect_file_type(ext: &str) -> (&'static str, bool) {
    match ext {
        "rs" | "py" | "js" | "ts" | "go" | "c" | "cpp" | "h" | "hpp" | "java" | "cs" | "rb"
        | "php" | "swift" | "kt" | "scala" | "sh" | "bash" | "zsh" | "fish" => ("code", true),
        "toml" | "yaml" | "yml" | "json" | "ini" | "xml" | "conf" | "config" | "env"
        | "properties" => ("config", true),
        "md" | "txt" | "rst" | "text" | "adoc" | "org" => ("text", true),
        _ => ("binary", false),
    }
}

/// 获取文件摘要（不读全文）
fn get_file_summary(path: &Path, ext: &str, size: u64) -> (String, String) {
    let (file_type, is_text) = detect_file_type(ext);

    if !is_text {
        return (file_type.to_string(), "[binary]".to_string());
    }

    // 文件过大，只返回类型信息
    if size > 512 * 1024 {
        // 512KB
        return (file_type.to_string(), format!("[large file: {}]", format_size(size)));
    }

    // 读取文件内容
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return (file_type.to_string(), "[read error]".to_string()),
    };

    let lines: Vec<&str> = content.lines().collect();

    if file_type == "code" {
        // 代码文件：提取结构关键词
        let preview = extract_code_structure(&lines, ext);
        (file_type.to_string(), preview)
    } else {
        // 配置/文本文件：取前 3 行摘要
        let preview: Vec<&str> = lines.iter().take(3).copied().collect();
        (file_type.to_string(), preview.join(" | ").chars().take(120).collect())
    }
}

/// 提取代码结构关键词（pub fn / struct / impl / mod 等）
fn extract_code_structure(lines: &[&str], ext: &str) -> String {
    let mut structure = Vec::new();

    for line in lines {
        let trimmed = line.trim();

        match ext {
            "rs" => {
                if trimmed.starts_with("pub fn ")
                    || trimmed.starts_with("fn ")
                    || trimmed.starts_with("pub struct ")
                    || trimmed.starts_with("struct ")
                    || trimmed.starts_with("pub impl ")
                    || trimmed.starts_with("impl ")
                    || trimmed.starts_with("pub mod ")
                    || trimmed.starts_with("mod ")
                    || trimmed.starts_with("pub trait ")
                    || trimmed.starts_with("trait ")
                    || trimmed.starts_with("pub enum ")
                    || trimmed.starts_with("enum ")
                {
                    // 只取签名部分（到 '{'、'{' 或行尾）
                    let sig = trimmed.split('{').next().unwrap_or(trimmed).trim_end();
                    structure.push(sig);
                    if structure.len() >= 8 {
                        break;
                    }
                }
            }
            "py" => {
                if trimmed.starts_with("def ")
                    || trimmed.starts_with("class ")
                    || trimmed.starts_with("async def ")
                {
                    let sig = trimmed.split(':').next().unwrap_or(trimmed).trim();
                    structure.push(sig);
                    if structure.len() >= 8 {
                        break;
                    }
                }
            }
            "js" | "ts" => {
                if trimmed.starts_with("function ")
                    || trimmed.starts_with("class ")
                    || trimmed.starts_with("const ")
                    || trimmed.starts_with("export ")
                    || trimmed.starts_with("async function ")
                {
                    let sig = trimmed.split('{').next().unwrap_or(trimmed).trim_end();
                    structure.push(sig);
                    if structure.len() >= 8 {
                        break;
                    }
                }
            }
            _ => {
                // 其他语言：取前 5 行非空非注释行
                if !trimmed.is_empty()
                    && !trimmed.starts_with("//")
                    && !trimmed.starts_with('#')
                    && !trimmed.starts_with("/*")
                    && !trimmed.starts_with('*')
                {
                    structure.push(trimmed);
                    if structure.len() >= 5 {
                        break;
                    }
                }
            }
        }
    }

    if structure.is_empty() {
        "[empty]".to_string()
    } else {
        structure.join("; ").chars().take(150).collect()
    }
}

/// 取文件头部 + 尾部预览（不读全文）
fn get_preview_from_lines(lines: &[&str], head: usize, tail: usize) -> String {
    let total = lines.len();

    if total <= head + tail {
        return lines.join("\n");
    }

    let mut out = Vec::new();
    out.extend(lines.iter().take(head).copied());
    out.push("...[truncated]...");
    out.extend(lines.iter().rev().take(tail).rev().copied());

    out.join("\n")
}

/// 代码文件压缩（去注释、空行，保留结构）
fn compress_code(content: &str, ext: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();

    for line in &lines {
        let trimmed = line.trim();

        // 跳过空行
        if trimmed.is_empty() {
            continue;
        }

        // 跳过单行注释
        if trimmed.starts_with("//")
            || trimmed.starts_with('#')
            || trimmed.starts_with('*')
            || trimmed.starts_with("/*")
        {
            continue;
        }

        match ext {
            "rs" => {
                // Rust：保留结构关键词
                if trimmed.starts_with("fn ")
                    || trimmed.starts_with("struct ")
                    || trimmed.starts_with("impl ")
                    || trimmed.starts_with("mod ")
                    || trimmed.starts_with("pub ")
                    || trimmed.starts_with("use ")
                    || trimmed.starts_with("trait ")
                    || trimmed.starts_with("enum ")
                    || trimmed.starts_with("type ")
                    || trimmed.starts_with("const ")
                    || trimmed.starts_with("let ")
                    || trimmed.starts_with("return ")
                    || trimmed.starts_with("}")
                {
                    result.push(*line);
                }
            }
            _ => {
                // 其他语言：保留非注释非空行
                result.push(*line);
            }
        }
    }

    result.join("\n")
}

/// 统计文件行数
fn count_lines(path: &Path) -> Option<usize> {
    let content = fs::read_to_string(path).ok()?;
    Some(content.lines().count())
}

/// 生成缓存 key
fn make_cache_key(path: &Path) -> String {
    let mtime = fs::metadata(path)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{}:{}", path.display(), mtime)
}

/// 格式化 SystemTime 为 ISO 8601
fn format_system_time(t: SystemTime) -> String {
    let secs = t.duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    // 简单格式化：转为可读时间戳
    let dt = chrono::DateTime::<chrono::Utc>::from_timestamp(secs as i64, 0)
        .unwrap_or(chrono::DateTime::<chrono::Utc>::UNIX_EPOCH);
    dt.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

/// 格式化过期时间
fn format_expires_at(timestamp: &Instant, ttl: Duration) -> String {
    let remaining = ttl.saturating_sub(timestamp.elapsed());
    format!("{}秒后过期", remaining.as_secs())
}

/// 格式化文件大小
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1}GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1}MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1}KB", bytes as f64 / KB as f64)
    } else {
        format!("{}B", bytes)
    }
}

/// 打印目录摘要
fn print_summary(summary: &DirSummary, output: &str) -> Result<()> {
    match output {
        "json" => print_json(summary),
        "tree" => print_tree(summary),
        "md" => print_markdown(summary),
        _ => print_json(summary),
    }
}

/// JSON 格式输出（默认）
fn print_json(summary: &DirSummary) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(summary)?);
    Ok(())
}

/// 树形格式输出
fn print_tree(summary: &DirSummary) -> Result<()> {
    println!("{} {}", "📁".cyan(), summary.root.cyan().bold());

    let mut dir_map: HashMap<String, Vec<&FileEntry>> = HashMap::new();

    for entry in &summary.files {
        let path = Path::new(&entry.path);
        let parent = path.parent().map(|p| p.display().to_string()).unwrap_or_default();
        dir_map.entry(parent).or_default().push(entry);
    }

    let mut sorted_dirs: Vec<&String> = dir_map.keys().collect();
    sorted_dirs.sort();

    for dir_name in sorted_dirs {
        if !dir_name.is_empty() {
            println!("  {} {}/", "├──".dimmed(), dir_name.yellow());
        }

        if let Some(entries) = dir_map.get(dir_name) {
            let mut sorted_entries = entries.to_vec();
            sorted_entries.sort_by(|a, b| a.path.cmp(&b.path));

            let count = sorted_entries.len();
            for (i, entry) in sorted_entries.iter().enumerate() {
                let is_last = i == count - 1;
                let prefix = if is_last { "  └──" } else { "  ├──" };
                let file_name = Path::new(&entry.path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(&entry.path);

                let type_icon = match entry.file_type.as_str() {
                    "code" => "📝",
                    "config" => "⚙️",
                    "text" => "📄",
                    _ => "📦",
                };

                let lines_str = entry.lines.map(|l| format!(" ({} 行)", l)).unwrap_or_default();

                println!(
                    "  {}{} {} {} [{} {}{}]",
                    if dir_name.is_empty() { "" } else { "   " },
                    prefix.dimmed(),
                    type_icon,
                    file_name.cyan(),
                    format_size(entry.size).yellow(),
                    entry.file_type.dimmed(),
                    lines_str.dimmed(),
                );
            }
        }
    }

    println!();
    println!(
        "{} {} 个文件，总大小 {}，耗时 {} ms",
        "✓".green(),
        summary.total_files.to_string().green().bold(),
        format_size(summary.total_size).cyan(),
        summary.elapsed_ms.to_string().yellow()
    );

    if let Some(ref cache_info) = summary.cache_info {
        if cache_info.cached {
            println!("  {} 使用缓存数据", "⚡".yellow());
        }
    }

    Ok(())
}

/// Markdown 表格格式输出
fn print_markdown(summary: &DirSummary) -> Result<()> {
    println!("# 目录摘要：{}", summary.root);
    println!();
    println!(
        "> 总计 {} 个文件，大小 {}，扫描耗时 {} ms",
        summary.total_files,
        format_size(summary.total_size),
        summary.elapsed_ms
    );
    println!();
    println!("| 路径 | 类型 | 大小 | 行数 | 摘要 |");
    println!("|------|------|------|------|------|");

    for entry in &summary.files {
        let lines_str = entry.lines.map(|l| l.to_string()).unwrap_or_else(|| "-".to_string());
        let summary_escaped = entry.summary.replace('|', "\\|");
        println!(
            "| `{}` | {} | {} | {} | {} |",
            entry.path,
            entry.file_type,
            format_size(entry.size),
            lines_str,
            summary_escaped
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_dir() -> TempDir {
        let dir = TempDir::new().unwrap();
        let src = dir.path().join("src");
        fs::create_dir(&src).unwrap();

        // 创建不同类型的文件
        fs::write(src.join("main.rs"), "fn main() {\n    println!(\"hello\");\n}\n").unwrap();
        fs::write(src.join("lib.rs"), "pub mod utils;\npub fn run() {}\n").unwrap();
        fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"test\"\n").unwrap();
        fs::write(dir.path().join("README.md"), "# Test Project\n\nThis is a test.\n").unwrap();

        dir
    }

    #[test]
    fn test_detect_file_type_code() {
        let (ty, is_text) = detect_file_type("rs");
        assert_eq!(ty, "code");
        assert!(is_text);

        let (ty, is_text) = detect_file_type("py");
        assert_eq!(ty, "code");
        assert!(is_text);
    }

    #[test]
    fn test_detect_file_type_config() {
        let (ty, is_text) = detect_file_type("toml");
        assert_eq!(ty, "config");
        assert!(is_text);

        let (ty, is_text) = detect_file_type("yaml");
        assert_eq!(ty, "config");
        assert!(is_text);
    }

    #[test]
    fn test_detect_file_type_binary() {
        let (ty, is_text) = detect_file_type("exe");
        assert_eq!(ty, "binary");
        assert!(!is_text);
    }

    #[test]
    fn test_extract_code_structure_rust() {
        let lines = vec![
            "use std::io;",
            "// comment",
            "pub struct MyStruct {",
            "    field: i32",
            "}",
            "",
            "impl MyStruct {",
            "    pub fn new() -> Self {",
        ];
        let result = extract_code_structure(&lines, "rs");
        assert!(result.contains("pub struct MyStruct"), "应包含 struct: {}", result);
        assert!(result.contains("impl MyStruct"), "应包含 impl: {}", result);
        assert!(result.contains("pub fn new"), "应包含 fn: {}", result);
    }

    #[test]
    fn test_extract_code_structure_python() {
        let lines = vec![
            "import os",
            "",
            "class MyClass:",
            "    def __init__(self):",
            "        pass",
            "",
            "def standalone_func():",
        ];
        let result = extract_code_structure(&lines, "py");
        assert!(result.contains("class MyClass"), "应包含 class: {}", result);
        assert!(result.contains("def"), "应包含 def: {}", result);
    }

    #[test]
    fn test_get_preview_from_lines_short() {
        let lines: Vec<&str> = (0..5).map(|_| "line").collect();
        let preview = get_preview_from_lines(&lines, 15, 10);
        assert_eq!(preview.lines().count(), 5);
        assert!(!preview.contains("[truncated]"));
    }

    #[test]
    fn test_get_preview_from_lines_long() {
        let lines: Vec<&str> = (0..100).map(|_| "line").collect();
        let preview = get_preview_from_lines(&lines, 15, 10);
        assert!(preview.contains("[truncated]"));
    }

    #[test]
    fn test_compress_code_rust() {
        let code = "// comment\nuse std::io;\n\npub fn main() {\n    let x = 1;\n}\n";
        let result = compress_code(code, "rs");
        assert!(!result.contains("// comment"), "注释应被去除");
        assert!(result.contains("pub fn main"), "pub fn 应保留");
        assert!(result.contains("use std::io"), "use 应保留");
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(500), "500B");
        assert_eq!(format_size(1024), "1.0KB");
        assert_eq!(format_size(1024 * 1024), "1.0MB");
    }

    #[test]
    fn test_count_lines() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("test.txt");
        fs::write(&file, "line1\nline2\nline3\n").unwrap();
        assert_eq!(count_lines(&file), Some(3));
    }

    #[test]
    fn test_scan_directory() {
        let dir = create_test_dir();
        let args = PeekArgs {
            directory: dir.path().display().to_string(),
            use_cache: false,
            ..Default::default()
        };

        let start = Instant::now();
        let summary = scan_directory(dir.path(), &args, "test-key", start).unwrap();

        assert!(summary.total_files > 0, "应扫描到文件");
        assert!(summary.total_size > 0, "总大小应大于 0");
        assert!(summary.elapsed_ms < 5000, "扫描不应超过 5 秒");

        // 验证文件条目
        let rs_files: Vec<_> = summary.files.iter().filter(|f| f.path.ends_with(".rs")).collect();
        assert!(!rs_files.is_empty(), "应找到 Rust 文件");
        assert!(rs_files.iter().all(|f| f.file_type == "code"), "Rust 文件应为 'code' 类型");
    }

    #[test]
    fn test_scan_directory_with_include_filter() {
        let dir = create_test_dir();
        let args = PeekArgs {
            directory: dir.path().display().to_string(),
            use_cache: false,
            include: Some("rs".to_string()),
            ..Default::default()
        };

        let start = Instant::now();
        let summary = scan_directory(dir.path(), &args, "test-key", start).unwrap();

        // 所有文件应为 .rs 类型
        assert!(summary.files.iter().all(|f| f.path.ends_with(".rs")), "应只有 .rs 文件");
    }

    #[test]
    fn test_json_output() {
        let dir = create_test_dir();
        let args = PeekArgs {
            directory: dir.path().display().to_string(),
            use_cache: false,
            output: "json".to_string(),
            ..Default::default()
        };

        // 应该不报错
        let result = execute(args);
        assert!(result.is_ok(), "JSON 输出应成功: {:?}", result);
    }

    #[test]
    fn test_tree_output() {
        let dir = create_test_dir();
        let args = PeekArgs {
            directory: dir.path().display().to_string(),
            use_cache: false,
            output: "tree".to_string(),
            ..Default::default()
        };

        let result = execute(args);
        assert!(result.is_ok(), "Tree 输出应成功: {:?}", result);
    }

    #[test]
    fn test_md_output() {
        let dir = create_test_dir();
        let args = PeekArgs {
            directory: dir.path().display().to_string(),
            use_cache: false,
            output: "md".to_string(),
            ..Default::default()
        };

        let result = execute(args);
        assert!(result.is_ok(), "Markdown 输出应成功: {:?}", result);
    }

    #[test]
    fn test_directory_not_exist() {
        let args =
            PeekArgs { directory: "/nonexistent/path/12345".to_string(), ..Default::default() };
        let result = execute(args);
        assert!(result.is_err(), "不存在的目录应报错");
    }

    #[test]
    fn test_peek_single_file() {
        let dir = create_test_dir();
        let args = PeekArgs {
            directory: dir.path().display().to_string(),
            file: Some("main.rs".to_string()),
            use_cache: false,
            output: "json".to_string(),
            ..Default::default()
        };

        let result = execute(args);
        assert!(result.is_ok(), "读取单个文件应成功: {:?}", result);
    }

    #[test]
    fn test_exclude_filter() {
        let dir = create_test_dir();
        // 创建一个 node_modules 目录
        let nm = dir.path().join("node_modules");
        fs::create_dir(&nm).unwrap();
        fs::write(nm.join("index.js"), "module.exports = {};").unwrap();

        let args = PeekArgs {
            directory: dir.path().display().to_string(),
            use_cache: false,
            output: "json".to_string(),
            ..Default::default()
        };

        let start = Instant::now();
        let summary = scan_directory(dir.path(), &args, "test-key", start).unwrap();

        // node_modules 内的文件不应被包含
        let nm_files: Vec<_> =
            summary.files.iter().filter(|f| f.path.contains("node_modules")).collect();
        assert!(nm_files.is_empty(), "node_modules 应被排除，但找到: {:?}", nm_files);
    }

    #[test]
    fn test_max_depth_filter() {
        let dir = create_test_dir();
        // 创建深层嵌套目录
        let deep = dir.path().join("a/b/c/d/e/f");
        fs::create_dir_all(&deep).unwrap();
        fs::write(deep.join("deep.rs"), "// deep file").unwrap();

        let args = PeekArgs {
            directory: dir.path().display().to_string(),
            use_cache: false,
            max_depth: Some(2),
            ..Default::default()
        };

        let start = Instant::now();
        let summary = scan_directory(dir.path(), &args, "test-key", start).unwrap();

        // 深层文件不应被包含
        let deep_files: Vec<_> =
            summary.files.iter().filter(|f| f.path.contains("a/b/c")).collect();
        assert!(deep_files.is_empty(), "超出深度限制的文件应被排除");
    }
}
