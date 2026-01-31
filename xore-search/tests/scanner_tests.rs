//! FileScanner 集成测试
//!
//! 测试文件扫描器的端到端功能，包括目录遍历、过滤和统计。

use std::fs::{self, File};
use std::io::Write;

use tempfile::TempDir;
use xore_search::{FileScanner, FileTypeFilter, MtimeFilter, ScanConfig, SizeFilter};

/// 创建测试目录结构
fn create_test_directory() -> TempDir {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let root = temp_dir.path();

    // 创建目录结构
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("data")).unwrap();
    fs::create_dir_all(root.join("logs")).unwrap();
    fs::create_dir_all(root.join(".hidden")).unwrap();

    // 创建各种类型的文件
    // Rust 源文件
    File::create(root.join("src/main.rs"))
        .unwrap()
        .write_all(b"fn main() { println!(\"Hello\"); }")
        .unwrap();
    File::create(root.join("src/lib.rs")).unwrap().write_all(b"pub mod utils;").unwrap();

    // CSV 数据文件
    File::create(root.join("data/users.csv"))
        .unwrap()
        .write_all(b"id,name,age\n1,Alice,30\n2,Bob,25")
        .unwrap();
    File::create(root.join("data/products.csv"))
        .unwrap()
        .write_all(b"id,name,price\n1,Apple,1.5\n2,Banana,0.8")
        .unwrap();

    // JSON 文件
    File::create(root.join("data/config.json"))
        .unwrap()
        .write_all(b"{\"key\": \"value\"}")
        .unwrap();

    // 日志文件
    File::create(root.join("logs/app.log"))
        .unwrap()
        .write_all(b"[INFO] Application started\n[ERROR] Something went wrong")
        .unwrap();

    // 隐藏目录中的文件
    File::create(root.join(".hidden/secret.txt")).unwrap().write_all(b"secret content").unwrap();

    // 大文件（用于大小过滤测试）
    let large_content = vec![b'x'; 10 * 1024]; // 10KB
    File::create(root.join("data/large.csv")).unwrap().write_all(&large_content).unwrap();

    // 创建 .gitignore
    File::create(root.join(".gitignore")).unwrap().write_all(b"*.log\n.hidden/").unwrap();

    temp_dir
}

#[test]
fn test_scan_all_files() {
    let temp_dir = create_test_directory();
    let config =
        ScanConfig::new(temp_dir.path()).with_include_hidden(true).with_respect_gitignore(false);

    let scanner = FileScanner::new(config);
    let (_files, stats) = scanner.scan().unwrap();

    // 应该找到所有文件（不包括目录和 .gitignore）
    assert!(stats.total_files >= 8, "Expected at least 8 files, got {}", stats.total_files);
    assert!(stats.directories > 0, "Expected some directories");
    assert!(stats.matched_files > 0, "Expected matched files");
}

#[test]
fn test_scan_with_type_filter() {
    let temp_dir = create_test_directory();
    let config = ScanConfig::new(temp_dir.path())
        .with_file_type(FileTypeFilter::Csv)
        .with_respect_gitignore(false);

    let scanner = FileScanner::new(config);
    let (files, stats) = scanner.scan().unwrap();

    // 应该只找到 CSV 文件
    assert_eq!(stats.matched_files, 3, "Expected 3 CSV files");
    for file in &files {
        assert!(
            file.path.extension().map(|e| e == "csv").unwrap_or(false),
            "Expected CSV file, got {:?}",
            file.path
        );
    }
}

#[test]
fn test_scan_with_code_filter() {
    let temp_dir = create_test_directory();
    let config = ScanConfig::new(temp_dir.path())
        .with_file_type(FileTypeFilter::Code)
        .with_respect_gitignore(false);

    let scanner = FileScanner::new(config);
    let (files, stats) = scanner.scan().unwrap();

    // 应该找到 Rust 源文件
    assert_eq!(stats.matched_files, 2, "Expected 2 code files");
    for file in &files {
        assert!(
            file.path.extension().map(|e| e == "rs").unwrap_or(false),
            "Expected .rs file, got {:?}",
            file.path
        );
    }
}

#[test]
fn test_scan_with_size_filter_greater_than() {
    let temp_dir = create_test_directory();
    let config = ScanConfig::new(temp_dir.path())
        .with_size_filter(SizeFilter::GreaterThan(5 * 1024)) // > 5KB
        .with_respect_gitignore(false);

    let scanner = FileScanner::new(config);
    let (files, stats) = scanner.scan().unwrap();

    // 应该只找到大于 5KB 的文件（large.csv 是 10KB）
    assert_eq!(stats.matched_files, 1, "Expected 1 large file");
    for file in &files {
        assert!(file.size > 5 * 1024, "Expected file > 5KB");
    }
}

#[test]
fn test_scan_with_size_filter_less_than() {
    let temp_dir = create_test_directory();
    let config = ScanConfig::new(temp_dir.path())
        .with_size_filter(SizeFilter::LessThan(1024)) // < 1KB
        .with_respect_gitignore(false);

    let scanner = FileScanner::new(config);
    let (files, stats) = scanner.scan().unwrap();

    // 大多数测试文件应该小于 1KB
    assert!(stats.matched_files > 0, "Expected some small files");
    for file in &files {
        assert!(file.size < 1024, "Expected file < 1KB, got {} bytes", file.size);
    }
}

#[test]
fn test_scan_with_size_range() {
    let temp_dir = create_test_directory();
    let config = ScanConfig::new(temp_dir.path())
        .with_size_filter(SizeFilter::Between(10, 500)) // 10B - 500B
        .with_respect_gitignore(false);

    let scanner = FileScanner::new(config);
    let (files, _stats) = scanner.scan().unwrap();

    for file in &files {
        assert!(
            file.size >= 10 && file.size <= 500,
            "Expected file size 10-500B, got {} bytes",
            file.size
        );
    }
}

#[test]
fn test_scan_with_gitignore() {
    let temp_dir = create_test_directory();

    // 初始化 git 仓库让 gitignore 生效
    std::process::Command::new("git").args(["init"]).current_dir(temp_dir.path()).output().ok();

    // 启用 gitignore（默认行为）
    let config =
        ScanConfig::new(temp_dir.path()).with_respect_gitignore(true).with_include_hidden(false);

    let scanner = FileScanner::new(config);
    let (files, _stats) = scanner.scan().unwrap();

    // .log 文件和 .hidden 目录应该被忽略
    for file in &files {
        let path_str = file.path.to_string_lossy();
        assert!(!path_str.ends_with(".log"), "Log file should be ignored: {:?}", file.path);
        assert!(
            !path_str.contains(".hidden"),
            "Hidden directory should be ignored: {:?}",
            file.path
        );
    }
}

#[test]
fn test_scan_include_hidden() {
    let temp_dir = create_test_directory();
    let config =
        ScanConfig::new(temp_dir.path()).with_include_hidden(true).with_respect_gitignore(false);

    let scanner = FileScanner::new(config);
    let (files, _stats) = scanner.scan().unwrap();

    // 应该能找到隐藏目录中的文件
    let has_hidden = files.iter().any(|f| f.path.to_string_lossy().contains(".hidden"));
    assert!(has_hidden, "Should find files in hidden directory");
}

#[test]
fn test_scan_max_depth() {
    let temp_dir = create_test_directory();

    // 只扫描第一层
    let config = ScanConfig::new(temp_dir.path()).with_max_depth(1).with_respect_gitignore(false);

    let scanner = FileScanner::new(config);
    let (files, _stats) = scanner.scan().unwrap();

    // 不应该找到子目录中的文件
    for file in &files {
        let relative_path = file.path.strip_prefix(temp_dir.path()).unwrap();
        let depth = relative_path.components().count();
        assert!(depth <= 1, "File depth should be <= 1, got {}: {:?}", depth, file.path);
    }
}

#[test]
fn test_scan_combined_filters() {
    let temp_dir = create_test_directory();

    // 组合多个过滤条件
    let config = ScanConfig::new(temp_dir.path())
        .with_file_type(FileTypeFilter::Csv)
        .with_size_filter(SizeFilter::LessThan(5 * 1024)) // < 5KB
        .with_respect_gitignore(false);

    let scanner = FileScanner::new(config);
    let (files, stats) = scanner.scan().unwrap();

    // 应该找到小于 5KB 的 CSV 文件
    for file in &files {
        assert!(file.path.extension().map(|e| e == "csv").unwrap_or(false), "Expected CSV file");
        assert!(file.size < 5 * 1024, "Expected file < 5KB");
    }
    // 排除 large.csv（10KB）
    assert_eq!(stats.matched_files, 2, "Expected 2 small CSV files");
}

#[test]
fn test_scan_custom_extensions() {
    let temp_dir = create_test_directory();

    // 使用自定义扩展名列表
    let config = ScanConfig::new(temp_dir.path())
        .with_file_type(FileTypeFilter::Custom(vec!["csv".to_string(), "json".to_string()]))
        .with_respect_gitignore(false);

    let scanner = FileScanner::new(config);
    let (files, stats) = scanner.scan().unwrap();

    // 应该找到 CSV 和 JSON 文件
    assert_eq!(stats.matched_files, 4, "Expected 4 files (3 CSV + 1 JSON)");
    for file in &files {
        let ext = file.path.extension().and_then(|e| e.to_str()).unwrap_or("");
        assert!(
            ext == "csv" || ext == "json",
            "Expected CSV or JSON, got .{} for {:?}",
            ext,
            file.path
        );
    }
}

#[test]
fn test_scan_statistics() {
    let temp_dir = create_test_directory();
    let config =
        ScanConfig::new(temp_dir.path()).with_respect_gitignore(false).with_include_hidden(true);

    let scanner = FileScanner::new(config);
    let (_files, stats) = scanner.scan().unwrap();

    // 验证统计信息合理性
    assert!(stats.total_files > 0, "Should have total files");
    assert!(stats.directories > 0, "Should have directories");
    assert!(stats.total_size > 0, "Should have total size");
    assert!(stats.elapsed_ms < 5000, "Scan should complete quickly");
    assert_eq!(
        stats.matched_files + stats.skipped,
        stats.total_files,
        "matched + skipped should equal total"
    );
}

#[test]
fn test_scan_empty_directory() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config = ScanConfig::new(temp_dir.path());

    let scanner = FileScanner::new(config);
    let (files, stats) = scanner.scan().unwrap();

    assert_eq!(files.len(), 0, "Empty directory should have no files");
    assert_eq!(stats.total_files, 0, "Empty directory should have no total files");
}

#[test]
fn test_mtime_filter_within_days() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // 创建一个新文件
    let new_file = temp_dir.path().join("new.txt");
    File::create(&new_file).unwrap().write_all(b"new content").unwrap();

    let config = ScanConfig::new(temp_dir.path()).with_mtime_filter(MtimeFilter::WithinDays(1)); // 过去1天内

    let scanner = FileScanner::new(config);
    let (files, stats) = scanner.scan().unwrap();

    // 刚创建的文件应该被找到
    assert_eq!(stats.matched_files, 1, "Should find newly created file");
    assert!(
        files[0].path.file_name().map(|n| n == "new.txt").unwrap_or(false),
        "Should be new.txt"
    );
}

#[test]
fn test_scan_iter() {
    let temp_dir = create_test_directory();
    let config = ScanConfig::new(temp_dir.path())
        .with_file_type(FileTypeFilter::Csv)
        .with_respect_gitignore(false);

    let scanner = FileScanner::new(config);
    let files: Vec<_> = scanner.scan_iter().filter_map(|r| r.ok()).collect();

    // 迭代器应该产生相同的结果
    assert_eq!(files.len(), 3, "Expected 3 CSV files from iterator");
}

#[test]
fn test_file_type_filter_parsing() {
    // 测试各种文件类型解析
    assert!(matches!(FileTypeFilter::parse("csv").unwrap(), FileTypeFilter::Csv));
    assert!(matches!(FileTypeFilter::parse("CSV").unwrap(), FileTypeFilter::Csv));
    assert!(matches!(FileTypeFilter::parse("json").unwrap(), FileTypeFilter::Json));
    assert!(matches!(FileTypeFilter::parse("jsonl").unwrap(), FileTypeFilter::Json));
    assert!(matches!(FileTypeFilter::parse("log").unwrap(), FileTypeFilter::Log));
    assert!(matches!(FileTypeFilter::parse("code").unwrap(), FileTypeFilter::Code));
    assert!(matches!(FileTypeFilter::parse("text").unwrap(), FileTypeFilter::Text));
    assert!(matches!(FileTypeFilter::parse("txt").unwrap(), FileTypeFilter::Text));
    assert!(matches!(FileTypeFilter::parse("parquet").unwrap(), FileTypeFilter::Parquet));
}

#[test]
fn test_size_filter_parsing() {
    // 测试各种大小过滤器解析
    assert!(matches!(SizeFilter::parse(">1KB").unwrap(), SizeFilter::GreaterThan(1024)));
    assert!(matches!(SizeFilter::parse("<10MB").unwrap(), SizeFilter::LessThan(10485760)));
    assert!(matches!(SizeFilter::parse("=1GB").unwrap(), SizeFilter::Equal(1073741824)));

    // 测试范围
    if let SizeFilter::Between(min, max) = SizeFilter::parse("1KB-1MB").unwrap() {
        assert_eq!(min, 1024);
        assert_eq!(max, 1024 * 1024);
    } else {
        panic!("Expected Between filter");
    }
}

#[test]
fn test_mtime_filter_parsing() {
    // 测试修改时间过滤器解析
    assert!(matches!(MtimeFilter::parse("-7d").unwrap(), MtimeFilter::WithinDays(7)));
    assert!(matches!(MtimeFilter::parse("+30d").unwrap(), MtimeFilter::OlderThanDays(30)));
    assert!(matches!(MtimeFilter::parse("2024-01-01").unwrap(), MtimeFilter::After(_)));

    // 测试无效格式
    assert!(MtimeFilter::parse("invalid").is_err());
}

#[test]
fn test_parallel_scan_threads() {
    let temp_dir = create_test_directory();

    // 测试指定线程数
    let config = ScanConfig::new(temp_dir.path()).with_threads(4).with_respect_gitignore(false);

    let scanner = FileScanner::new(config);
    let (_files, stats) = scanner.scan().unwrap();

    // 应该正常工作
    assert!(stats.total_files > 0, "Should find files with 4 threads");
}
