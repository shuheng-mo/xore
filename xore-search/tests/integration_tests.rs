//! 集成测试 - 验证各模块协同工作

use std::fs;
use tempfile::TempDir;
use xore_search::{FileScanner, IndexBuilder, IndexConfig, ScanConfig, Searcher};

#[test]
fn test_end_to_end_indexing_and_searching() {
    // 创建临时目录和测试文件
    let temp_dir = TempDir::new().unwrap();

    // 创建测试文件
    let file1 = temp_dir.path().join("test1.txt");
    fs::write(&file1, "hello world rust programming").unwrap();

    let file2 = temp_dir.path().join("test2.txt");
    fs::write(&file2, "rust error handling with anyhow").unwrap();

    let file3 = temp_dir.path().join("test3.md");
    fs::write(&file3, "# Rust Documentation\nThis is a test").unwrap();

    // 1. 扫描文件
    let scan_config = ScanConfig::new(temp_dir.path());
    let scanner = FileScanner::new(scan_config);
    let (files, stats) = scanner.scan().unwrap();

    assert!(files.len() >= 3);
    assert_eq!(stats.total_files, files.len());

    // 2. 构建索引
    let index_path = temp_dir.path().join("index");
    let index_config = IndexConfig { index_path: index_path.clone(), ..Default::default() };

    let mut builder = IndexBuilder::with_config(index_config).unwrap();
    let added = builder.add_documents_batch(&files).unwrap();
    assert!(added >= 3);

    let index_stats = builder.build().unwrap();
    assert!(index_stats.documents_added >= 3);

    // 3. 搜索
    let searcher = Searcher::new(&index_path).unwrap();

    // 搜索 "rust"
    let results = searcher.search("rust").unwrap();
    assert!(!results.is_empty(), "Should find matches for 'rust'");

    // 搜索 "error"
    let results = searcher.search("error").unwrap();
    assert!(!results.is_empty(), "Should find matches for 'error'");

    // 搜索不存在的词
    let results = searcher.search("nonexistent").unwrap();
    assert!(results.is_empty(), "Should not find matches for nonexistent word");
}

#[test]
fn test_file_type_filtering() {
    let temp_dir = TempDir::new().unwrap();

    // 创建不同类型的文件
    fs::write(temp_dir.path().join("test.rs"), "fn main() {}").unwrap();
    fs::write(temp_dir.path().join("test.md"), "# Title").unwrap();
    fs::write(temp_dir.path().join("test.txt"), "content").unwrap();

    // 扫描并索引
    let index_path = temp_dir.path().join("index");
    let scan_config = ScanConfig::new(temp_dir.path());
    let scanner = FileScanner::new(scan_config);
    let (files, _) = scanner.scan().unwrap();

    let index_config = IndexConfig { index_path: index_path.clone(), ..Default::default() };
    let mut builder = IndexBuilder::with_config(index_config).unwrap();
    builder.add_documents_batch(&files).unwrap();
    builder.build().unwrap();

    // 搜索特定文件类型
    let searcher = Searcher::new(&index_path).unwrap();

    // 由于当前实现没有文件类型过滤API，这里只验证基本搜索
    let results = searcher.search("main").unwrap();
    assert!(!results.is_empty());
}

#[test]
fn test_chinese_text_search() {
    let temp_dir = TempDir::new().unwrap();

    // 创建包含中文的文件
    let file1 = temp_dir.path().join("chinese.txt");
    fs::write(&file1, "这是一个测试文件，包含中文内容").unwrap();

    let file2 = temp_dir.path().join("mixed.txt");
    fs::write(&file2, "This file contains 中英文混合 content").unwrap();

    // 扫描并索引
    let index_path = temp_dir.path().join("index");
    let scan_config = ScanConfig::new(temp_dir.path());
    let scanner = FileScanner::new(scan_config);
    let (files, _) = scanner.scan().unwrap();

    let index_config = IndexConfig { index_path: index_path.clone(), ..Default::default() };
    let mut builder = IndexBuilder::with_config(index_config).unwrap();
    builder.add_documents_batch(&files).unwrap();
    builder.build().unwrap();

    // 搜索中文
    let searcher = Searcher::new(&index_path).unwrap();
    let results = searcher.search("测试").unwrap();
    assert!(!results.is_empty(), "Should find Chinese text");

    // 搜索混合内容
    let results = searcher.search("中英文").unwrap();
    assert!(!results.is_empty(), "Should find mixed text");
}

#[test]
fn test_incremental_update() {
    let temp_dir = TempDir::new().unwrap();

    // 创建初始文件
    let file1 = temp_dir.path().join("test.txt");
    fs::write(&file1, "initial content").unwrap();

    // 初始索引
    let index_path = temp_dir.path().join("index");
    let scan_config = ScanConfig::new(temp_dir.path());
    let scanner = FileScanner::new(scan_config);
    let (files, _) = scanner.scan().unwrap();

    let index_config = IndexConfig { index_path: index_path.clone(), ..Default::default() };
    let mut builder = IndexBuilder::with_config(index_config).unwrap();
    builder.add_documents_batch(&files).unwrap();
    builder.build().unwrap();

    // 搜索初始内容
    let searcher = Searcher::new(&index_path).unwrap();
    let results = searcher.search("initial").unwrap();
    assert!(!results.is_empty());

    // 修改文件
    fs::write(&file1, "updated content").unwrap();

    // 重新索引（模拟增量更新）
    let scan_config = ScanConfig::new(temp_dir.path());
    let scanner = FileScanner::new(scan_config);
    let (files, _) = scanner.scan().unwrap();

    let index_config = IndexConfig { index_path: index_path.clone(), ..Default::default() };
    let mut builder = IndexBuilder::with_config(index_config).unwrap();
    builder.add_documents_batch(&files).unwrap();
    builder.build().unwrap();

    // 搜索新内容
    let searcher = Searcher::new(&index_path).unwrap();
    let results = searcher.search("updated").unwrap();
    assert!(!results.is_empty(), "Should find updated content");
}
