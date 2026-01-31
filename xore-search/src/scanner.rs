//! 文件扫描器模块
//!
//! 基于 `walkdir` + `ignore` 实现高性能文件扫描，支持：
//! - 遵守 .gitignore 规则
//! - Rayon 并行遍历
//! - 多种过滤条件（类型、大小、修改时间）

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use anyhow::{Context, Result};
use ignore::WalkBuilder;
use rayon::prelude::*;
use tracing::{debug, info, instrument, warn};

/// 文件大小过滤条件
#[derive(Debug, Clone)]
pub enum SizeFilter {
    /// 大于指定字节数
    GreaterThan(u64),
    /// 小于指定字节数
    LessThan(u64),
    /// 等于指定字节数
    Equal(u64),
    /// 在指定范围内（包含边界）
    Between(u64, u64),
}

impl SizeFilter {
    /// 解析大小过滤字符串
    /// 支持格式：">1MB", "<500KB", "=1GB", "1MB-10MB"
    pub fn parse(s: &str) -> Result<Self> {
        let s = s.trim();

        // 处理范围格式 "1MB-10MB"
        if let Some((min, max)) = s.split_once('-') {
            let min_bytes = parse_size(min.trim())?;
            let max_bytes = parse_size(max.trim())?;
            return Ok(SizeFilter::Between(min_bytes, max_bytes));
        }

        // 处理比较格式
        if let Some(rest) = s.strip_prefix('>') {
            let bytes = parse_size(rest.trim())?;
            Ok(SizeFilter::GreaterThan(bytes))
        } else if let Some(rest) = s.strip_prefix('<') {
            let bytes = parse_size(rest.trim())?;
            Ok(SizeFilter::LessThan(bytes))
        } else if let Some(rest) = s.strip_prefix('=') {
            let bytes = parse_size(rest.trim())?;
            Ok(SizeFilter::Equal(bytes))
        } else {
            // 默认等于
            let bytes = parse_size(s)?;
            Ok(SizeFilter::Equal(bytes))
        }
    }

    /// 检查文件大小是否匹配过滤条件
    pub fn matches(&self, size: u64) -> bool {
        match self {
            SizeFilter::GreaterThan(threshold) => size > *threshold,
            SizeFilter::LessThan(threshold) => size < *threshold,
            SizeFilter::Equal(target) => size == *target,
            SizeFilter::Between(min, max) => size >= *min && size <= *max,
        }
    }
}

/// 修改时间过滤条件
#[derive(Debug, Clone)]
pub enum MtimeFilter {
    /// 在指定时间之后修改
    After(SystemTime),
    /// 在指定时间之前修改
    Before(SystemTime),
    /// 在过去N天内修改
    WithinDays(u32),
    /// 超过N天未修改
    OlderThanDays(u32),
}

impl MtimeFilter {
    /// 解析修改时间过滤字符串
    /// 支持格式："-7d"（过去7天）, "+30d"（超过30天）, "2024-01-01"（指定日期之后）
    pub fn parse(s: &str) -> Result<Self> {
        let s = s.trim();

        // 处理相对时间格式
        if let Some(rest) = s.strip_prefix('-') {
            if let Some(days_str) = rest.strip_suffix('d') {
                let days: u32 = days_str.parse().context("Invalid days number")?;
                return Ok(MtimeFilter::WithinDays(days));
            }
        }

        if let Some(rest) = s.strip_prefix('+') {
            if let Some(days_str) = rest.strip_suffix('d') {
                let days: u32 = days_str.parse().context("Invalid days number")?;
                return Ok(MtimeFilter::OlderThanDays(days));
            }
        }

        // 处理绝对日期格式 YYYY-MM-DD
        if let Ok(date) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
            let datetime = date.and_hms_opt(0, 0, 0).context("Invalid date")?;
            let system_time =
                SystemTime::UNIX_EPOCH + Duration::from_secs(datetime.and_utc().timestamp() as u64);
            return Ok(MtimeFilter::After(system_time));
        }

        anyhow::bail!("Invalid mtime filter format: {}. Use '-7d', '+30d', or 'YYYY-MM-DD'", s)
    }

    /// 检查修改时间是否匹配过滤条件
    pub fn matches(&self, mtime: SystemTime) -> bool {
        let now = SystemTime::now();

        match self {
            MtimeFilter::After(threshold) => mtime >= *threshold,
            MtimeFilter::Before(threshold) => mtime <= *threshold,
            MtimeFilter::WithinDays(days) => {
                let threshold = now - Duration::from_secs(*days as u64 * 24 * 60 * 60);
                mtime >= threshold
            }
            MtimeFilter::OlderThanDays(days) => {
                let threshold = now - Duration::from_secs(*days as u64 * 24 * 60 * 60);
                mtime < threshold
            }
        }
    }
}

/// 文件类型过滤
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileTypeFilter {
    Csv,
    Json,
    Log,
    Code,
    Text,
    Parquet,
    Custom(Vec<String>),
}

impl FileTypeFilter {
    /// 解析文件类型过滤字符串
    pub fn parse(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "csv" => Ok(FileTypeFilter::Csv),
            "json" | "jsonl" => Ok(FileTypeFilter::Json),
            "log" => Ok(FileTypeFilter::Log),
            "code" => Ok(FileTypeFilter::Code),
            "text" | "txt" => Ok(FileTypeFilter::Text),
            "parquet" => Ok(FileTypeFilter::Parquet),
            _ => {
                // 支持逗号分隔的扩展名列表
                let extensions: Vec<String> = s
                    .split(',')
                    .map(|ext| ext.trim().trim_start_matches('.').to_lowercase())
                    .collect();
                Ok(FileTypeFilter::Custom(extensions))
            }
        }
    }

    /// 获取该类型对应的文件扩展名列表
    pub fn extensions(&self) -> Vec<&str> {
        match self {
            FileTypeFilter::Csv => vec!["csv", "tsv"],
            FileTypeFilter::Json => vec!["json", "jsonl", "ndjson"],
            FileTypeFilter::Log => vec!["log", "logs"],
            FileTypeFilter::Code => vec![
                "rs", "py", "js", "ts", "go", "java", "c", "cpp", "h", "hpp", "rb", "php", "swift",
                "kt", "scala", "sh", "bash", "zsh",
            ],
            FileTypeFilter::Text => vec!["txt", "md", "rst", "text"],
            FileTypeFilter::Parquet => vec!["parquet", "pq"],
            FileTypeFilter::Custom(exts) => exts.iter().map(|s| s.as_str()).collect(),
        }
    }

    /// 检查文件是否匹配类型过滤
    pub fn matches(&self, path: &Path) -> bool {
        let ext = path.extension().and_then(|e| e.to_str()).map(|e| e.to_lowercase());

        match ext {
            Some(ext) => self.extensions().contains(&ext.as_str()),
            None => false,
        }
    }
}

/// 扫描到的文件信息
#[derive(Debug, Clone)]
pub struct ScannedFile {
    /// 文件路径
    pub path: PathBuf,
    /// 文件大小（字节）
    pub size: u64,
    /// 修改时间
    pub modified: Option<SystemTime>,
    /// 是否为目录
    pub is_dir: bool,
}

impl ScannedFile {
    /// 从 DirEntry 创建 ScannedFile
    fn from_entry(entry: &ignore::DirEntry) -> Option<Self> {
        let metadata = entry.metadata().ok()?;
        Some(ScannedFile {
            path: entry.path().to_path_buf(),
            size: metadata.len(),
            modified: metadata.modified().ok(),
            is_dir: metadata.is_dir(),
        })
    }
}

/// 扫描配置
#[derive(Debug, Clone)]
pub struct ScanConfig {
    /// 扫描根路径
    pub root: PathBuf,
    /// 文件类型过滤
    pub file_type: Option<FileTypeFilter>,
    /// 文件大小过滤
    pub size_filter: Option<SizeFilter>,
    /// 修改时间过滤
    pub mtime_filter: Option<MtimeFilter>,
    /// 最大遍历深度（None 表示无限制）
    pub max_depth: Option<usize>,
    /// 是否跟随符号链接
    pub follow_links: bool,
    /// 是否遵守 .gitignore
    pub respect_gitignore: bool,
    /// 并行线程数（0 表示自动检测）
    pub threads: usize,
    /// 是否包含隐藏文件
    pub include_hidden: bool,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            root: PathBuf::from("."),
            file_type: None,
            size_filter: None,
            mtime_filter: None,
            max_depth: None,
            follow_links: false,
            respect_gitignore: true,
            threads: 0, // 自动检测
            include_hidden: false,
        }
    }
}

impl ScanConfig {
    /// 创建新的扫描配置
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        Self { root: root.as_ref().to_path_buf(), ..Default::default() }
    }

    /// 设置文件类型过滤
    pub fn with_file_type(mut self, filter: FileTypeFilter) -> Self {
        self.file_type = Some(filter);
        self
    }

    /// 设置文件大小过滤
    pub fn with_size_filter(mut self, filter: SizeFilter) -> Self {
        self.size_filter = Some(filter);
        self
    }

    /// 设置修改时间过滤
    pub fn with_mtime_filter(mut self, filter: MtimeFilter) -> Self {
        self.mtime_filter = Some(filter);
        self
    }

    /// 设置最大深度
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = Some(depth);
        self
    }

    /// 设置是否跟随符号链接
    pub fn with_follow_links(mut self, follow: bool) -> Self {
        self.follow_links = follow;
        self
    }

    /// 设置是否遵守 .gitignore
    pub fn with_respect_gitignore(mut self, respect: bool) -> Self {
        self.respect_gitignore = respect;
        self
    }

    /// 设置并行线程数
    pub fn with_threads(mut self, threads: usize) -> Self {
        self.threads = threads;
        self
    }

    /// 设置是否包含隐藏文件
    pub fn with_include_hidden(mut self, include: bool) -> Self {
        self.include_hidden = include;
        self
    }
}

/// 扫描统计信息
#[derive(Debug, Clone, Default)]
pub struct ScanStats {
    /// 扫描的文件总数
    pub total_files: usize,
    /// 匹配过滤条件的文件数
    pub matched_files: usize,
    /// 扫描的目录数
    pub directories: usize,
    /// 跳过的文件数（因过滤条件）
    pub skipped: usize,
    /// 错误数
    pub errors: usize,
    /// 总文件大小（字节）
    pub total_size: u64,
    /// 扫描耗时（毫秒）
    pub elapsed_ms: u64,
}

/// 文件扫描器
pub struct FileScanner {
    config: ScanConfig,
}

impl FileScanner {
    /// 创建新的文件扫描器
    pub fn new(config: ScanConfig) -> Self {
        Self { config }
    }

    /// 构建 WalkBuilder
    fn build_walker(&self) -> WalkBuilder {
        let mut builder = WalkBuilder::new(&self.config.root);

        // 配置并行线程数
        let threads = if self.config.threads == 0 { num_cpus::get() } else { self.config.threads };
        builder.threads(threads);

        // 配置 gitignore
        builder.git_ignore(self.config.respect_gitignore);
        builder.git_global(self.config.respect_gitignore);
        builder.git_exclude(self.config.respect_gitignore);

        // 配置隐藏文件
        builder.hidden(!self.config.include_hidden);

        // 配置符号链接
        builder.follow_links(self.config.follow_links);

        // 配置最大深度
        if let Some(depth) = self.config.max_depth {
            builder.max_depth(Some(depth));
        }

        // 添加标准忽略文件
        builder.add_custom_ignore_filename(".xoreignore");

        builder
    }

    /// 检查文件是否匹配所有过滤条件
    fn matches_filters(&self, file: &ScannedFile) -> bool {
        // 跳过目录
        if file.is_dir {
            return false;
        }

        // 检查文件类型
        if let Some(ref filter) = self.config.file_type {
            if !filter.matches(&file.path) {
                return false;
            }
        }

        // 检查文件大小
        if let Some(ref filter) = self.config.size_filter {
            if !filter.matches(file.size) {
                return false;
            }
        }

        // 检查修改时间
        if let Some(ref filter) = self.config.mtime_filter {
            if let Some(mtime) = file.modified {
                if !filter.matches(mtime) {
                    return false;
                }
            } else {
                // 无法获取修改时间，跳过
                return false;
            }
        }

        true
    }

    /// 执行扫描（并行版本）
    #[instrument(skip(self), fields(root = %self.config.root.display()))]
    pub fn scan(&self) -> Result<(Vec<ScannedFile>, ScanStats)> {
        let start = std::time::Instant::now();
        let mut stats = ScanStats::default();

        info!("Starting file scan at {:?}", self.config.root);
        debug!("Scan config: {:?}", self.config);

        let walker = self.build_walker();

        // 收集所有条目
        let entries: Vec<_> = walker
            .build()
            .filter_map(|entry| match entry {
                Ok(e) => Some(e),
                Err(err) => {
                    warn!("Error accessing entry: {}", err);
                    None
                }
            })
            .collect();

        // 使用 Rayon 并行处理
        let results: Vec<(Option<ScannedFile>, bool, bool)> = entries
            .par_iter()
            .map(|entry| {
                let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);

                match ScannedFile::from_entry(entry) {
                    Some(file) => {
                        let matches = self.matches_filters(&file);
                        (Some(file), is_dir, matches)
                    }
                    None => (None, is_dir, false),
                }
            })
            .collect();

        // 汇总结果
        let mut matched_files = Vec::new();

        for (file, is_dir, matches) in results {
            if is_dir {
                stats.directories += 1;
            } else {
                stats.total_files += 1;
            }

            if let Some(f) = file {
                stats.total_size += f.size;

                if matches {
                    stats.matched_files += 1;
                    matched_files.push(f);
                } else if !is_dir {
                    stats.skipped += 1;
                }
            }
        }

        stats.elapsed_ms = start.elapsed().as_millis() as u64;

        info!(
            "Scan completed: {} files matched out of {} total ({} ms)",
            stats.matched_files, stats.total_files, stats.elapsed_ms
        );

        Ok((matched_files, stats))
    }

    /// 执行扫描并返回迭代器（内存友好版本）
    pub fn scan_iter(&self) -> impl Iterator<Item = Result<ScannedFile>> + '_ {
        let walker = self.build_walker();

        walker.build().filter_map(move |entry| match entry {
            Ok(e) => {
                let file = ScannedFile::from_entry(&e)?;
                if self.matches_filters(&file) {
                    Some(Ok(file))
                } else {
                    None
                }
            }
            Err(err) => Some(Err(anyhow::anyhow!("Error accessing entry: {}", err))),
        })
    }
}

/// 解析大小字符串为字节数
/// 支持格式：1024, 1KB, 1.5MB, 2GB
fn parse_size(s: &str) -> Result<u64> {
    let s = s.trim().to_uppercase();

    // 尝试直接解析为数字（纯字节）
    if let Ok(bytes) = s.parse::<u64>() {
        return Ok(bytes);
    }

    // 解析带单位的格式
    let (num_str, unit) = if s.ends_with("GB") {
        (&s[..s.len() - 2], 1024 * 1024 * 1024)
    } else if s.ends_with("MB") {
        (&s[..s.len() - 2], 1024 * 1024)
    } else if s.ends_with("KB") {
        (&s[..s.len() - 2], 1024)
    } else if s.ends_with('B') {
        (&s[..s.len() - 1], 1)
    } else {
        return Err(anyhow::anyhow!(
            "Invalid size format: {}. Use format like '1MB', '500KB', '2GB'",
            s
        ));
    };

    let num: f64 =
        num_str.trim().parse().with_context(|| format!("Invalid number in size: {}", num_str))?;

    Ok((num * unit as f64) as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    mod size_filter_tests {
        use super::*;

        #[test]
        fn test_parse_bytes() {
            let filter = SizeFilter::parse("1024").unwrap();
            assert!(matches!(filter, SizeFilter::Equal(1024)));
        }

        #[test]
        fn test_parse_kb() {
            let filter = SizeFilter::parse(">1KB").unwrap();
            assert!(matches!(filter, SizeFilter::GreaterThan(1024)));
        }

        #[test]
        fn test_parse_mb() {
            let filter = SizeFilter::parse("<10MB").unwrap();
            assert!(matches!(filter, SizeFilter::LessThan(10485760)));
        }

        #[test]
        fn test_parse_range() {
            let filter = SizeFilter::parse("1MB-10MB").unwrap();
            match filter {
                SizeFilter::Between(min, max) => {
                    assert_eq!(min, 1024 * 1024);
                    assert_eq!(max, 10 * 1024 * 1024);
                }
                _ => panic!("Expected Between filter"),
            }
        }

        #[test]
        fn test_matches() {
            assert!(SizeFilter::GreaterThan(100).matches(200));
            assert!(!SizeFilter::GreaterThan(100).matches(50));
            assert!(SizeFilter::LessThan(100).matches(50));
            assert!(SizeFilter::Between(10, 100).matches(50));
            assert!(!SizeFilter::Between(10, 100).matches(5));
        }
    }

    mod mtime_filter_tests {
        use super::*;

        #[test]
        fn test_parse_within_days() {
            let filter = MtimeFilter::parse("-7d").unwrap();
            assert!(matches!(filter, MtimeFilter::WithinDays(7)));
        }

        #[test]
        fn test_parse_older_than_days() {
            let filter = MtimeFilter::parse("+30d").unwrap();
            assert!(matches!(filter, MtimeFilter::OlderThanDays(30)));
        }

        #[test]
        fn test_parse_date() {
            let filter = MtimeFilter::parse("2024-01-01").unwrap();
            assert!(matches!(filter, MtimeFilter::After(_)));
        }

        #[test]
        fn test_within_days_matches() {
            let filter = MtimeFilter::WithinDays(7);
            let recent = SystemTime::now() - Duration::from_secs(3 * 24 * 60 * 60);
            let old = SystemTime::now() - Duration::from_secs(10 * 24 * 60 * 60);

            assert!(filter.matches(recent));
            assert!(!filter.matches(old));
        }
    }

    mod file_type_filter_tests {
        use super::*;

        #[test]
        fn test_parse_csv() {
            let filter = FileTypeFilter::parse("csv").unwrap();
            assert_eq!(filter, FileTypeFilter::Csv);
        }

        #[test]
        fn test_parse_custom() {
            let filter = FileTypeFilter::parse("xml,yaml,toml").unwrap();
            match filter {
                FileTypeFilter::Custom(exts) => {
                    assert_eq!(exts, vec!["xml", "yaml", "toml"]);
                }
                _ => panic!("Expected Custom filter"),
            }
        }

        #[test]
        fn test_matches_csv() {
            let filter = FileTypeFilter::Csv;
            assert!(filter.matches(Path::new("data.csv")));
            assert!(filter.matches(Path::new("data.tsv")));
            assert!(!filter.matches(Path::new("data.json")));
        }

        #[test]
        fn test_matches_code() {
            let filter = FileTypeFilter::Code;
            assert!(filter.matches(Path::new("main.rs")));
            assert!(filter.matches(Path::new("app.py")));
            assert!(!filter.matches(Path::new("data.csv")));
        }
    }

    mod parse_size_tests {
        use super::*;

        #[test]
        fn test_parse_pure_bytes() {
            assert_eq!(parse_size("1024").unwrap(), 1024);
        }

        #[test]
        fn test_parse_kb() {
            assert_eq!(parse_size("1KB").unwrap(), 1024);
            assert_eq!(parse_size("1kb").unwrap(), 1024);
        }

        #[test]
        fn test_parse_mb() {
            assert_eq!(parse_size("1MB").unwrap(), 1024 * 1024);
        }

        #[test]
        fn test_parse_gb() {
            assert_eq!(parse_size("1GB").unwrap(), 1024 * 1024 * 1024);
        }

        #[test]
        fn test_parse_decimal() {
            assert_eq!(parse_size("1.5MB").unwrap(), (1.5 * 1024.0 * 1024.0) as u64);
        }
    }
}
