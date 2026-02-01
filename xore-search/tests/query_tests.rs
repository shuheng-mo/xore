//! 搜索查询测试
//!
//! 测试覆盖：
//! - 中文关键词搜索
//! - 英文关键词搜索
//! - 中英混合搜索
//! - 短语搜索
//! - 多词搜索
//! - 类型过滤搜索
//! - 空结果处理
//! - 结果排序验证

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use tempfile::TempDir;
use xore_search::indexer::IndexBuilder;
use xore_search::query::{SearchConfig, Searcher};
use xore_search::scanner::ScannedFile;

/// 创建测试文件
fn create_test_file(dir: &Path, name: &str, content: &str) -> PathBuf {
    let path = dir.join(name);
    let mut file = File::create(&path).unwrap();
    file.write_all(content.as_bytes()).unwrap();
    path
}

/// 创建 ScannedFile 结构
fn scanned_file(path: PathBuf, content: &str) -> ScannedFile {
    ScannedFile {
        path,
        size: content.len() as u64,
        modified: Some(SystemTime::now()),
        is_dir: false,
    }
}

/// 设置测试索引，返回索引路径
fn setup_test_index(temp_dir: &TempDir) -> PathBuf {
    let index_path = temp_dir.path().join("test_index");
    let files_dir = temp_dir.path().join("files");
    std::fs::create_dir_all(&files_dir).unwrap();

    // 创建各种测试文件
    let test_files = vec![
        ("error.log", "This is an error message\nAnother line with error\nERROR: critical failure"),
        ("chinese.txt", "这是一个错误日志\n数据处理完成\n测试文件"),
        ("mixed.txt", "Error 错误 processing data 数据处理\n混合内容 mixed content"),
        ("hello.rs", "fn main() {\n    println!(\"Hello, world!\");\n    // 注释\n}\n"),
        ("data.json", "{\"name\": \"test\", \"value\": 123, \"error\": false}"),
        ("readme.md", "# Project README\n\nThis is a sample project.\n\n## Features\n- Feature one\n- Feature two"),
        ("config.log", "INFO: Starting application\nWARN: Configuration missing\nERROR: Failed to connect"),
    ];

    let mut scanned_files = Vec::new();
    for (name, content) in test_files {
        let path = create_test_file(&files_dir, name, content);
        scanned_files.push(scanned_file(path, content));
    }

    // 构建索引
    let mut builder = IndexBuilder::new(&index_path).unwrap();
    builder.add_documents_batch(&scanned_files).unwrap();
    builder.build().unwrap();

    index_path
}

mod english_search_tests {
    use super::*;

    #[test]
    fn test_search_single_word() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = setup_test_index(&temp_dir);

        let searcher = Searcher::new(&index_path).unwrap();
        let results = searcher.search("error").unwrap();

        assert!(!results.is_empty());
        // 应该找到 error.log, config.log, data.json
        assert!(results.iter().any(|r| r.path.to_string_lossy().contains("error.log")));
    }

    #[test]
    fn test_search_case_insensitive() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = setup_test_index(&temp_dir);

        let searcher = Searcher::new(&index_path).unwrap();

        let results_lower = searcher.search("error").unwrap();
        let results_upper = searcher.search("ERROR").unwrap();
        let results_mixed = searcher.search("Error").unwrap();

        // 大小写不敏感，结果应该相同
        assert_eq!(results_lower.len(), results_upper.len());
        assert_eq!(results_lower.len(), results_mixed.len());
    }

    #[test]
    fn test_search_hello() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = setup_test_index(&temp_dir);

        let searcher = Searcher::new(&index_path).unwrap();
        let results = searcher.search("hello").unwrap();

        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.path.to_string_lossy().contains("hello.rs")));
    }

    #[test]
    fn test_search_readme() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = setup_test_index(&temp_dir);

        let searcher = Searcher::new(&index_path).unwrap();
        let results = searcher.search("project").unwrap();

        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.path.to_string_lossy().contains("readme.md")));
    }
}

mod chinese_search_tests {
    use super::*;

    #[test]
    fn test_search_chinese_word() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = setup_test_index(&temp_dir);

        let searcher = Searcher::new(&index_path).unwrap();
        let results = searcher.search("错误").unwrap();

        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.path.to_string_lossy().contains("chinese.txt")));
    }

    #[test]
    fn test_search_chinese_data() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = setup_test_index(&temp_dir);

        let searcher = Searcher::new(&index_path).unwrap();
        let results = searcher.search("数据").unwrap();

        assert!(!results.is_empty());
    }

    #[test]
    fn test_search_chinese_in_mixed_file() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = setup_test_index(&temp_dir);

        let searcher = Searcher::new(&index_path).unwrap();
        let results = searcher.search("混合").unwrap();

        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.path.to_string_lossy().contains("mixed.txt")));
    }
}

mod mixed_search_tests {
    use super::*;

    #[test]
    fn test_search_mixed_query() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = setup_test_index(&temp_dir);

        let searcher = Searcher::new(&index_path).unwrap();

        // 搜索中英文混合查询
        let results = searcher.search("error 错误").unwrap();

        // 应该找到包含 error 或 错误 的文件
        assert!(!results.is_empty());
    }

    #[test]
    fn test_search_mixed_content_file() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = setup_test_index(&temp_dir);

        let searcher = Searcher::new(&index_path).unwrap();
        let results = searcher.search("processing").unwrap();

        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.path.to_string_lossy().contains("mixed.txt")));
    }
}

mod phrase_search_tests {
    use super::*;

    #[test]
    fn test_phrase_search() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = setup_test_index(&temp_dir);

        let searcher = Searcher::new(&index_path).unwrap();

        // 短语搜索使用双引号
        let results = searcher.search("\"Hello, world\"").unwrap();

        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.path.to_string_lossy().contains("hello.rs")));
    }

    #[test]
    fn test_phrase_search_error_message() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = setup_test_index(&temp_dir);

        let searcher = Searcher::new(&index_path).unwrap();
        let results = searcher.search("\"error message\"").unwrap();

        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.path.to_string_lossy().contains("error.log")));
    }
}

mod multi_word_search_tests {
    use super::*;

    #[test]
    fn test_multi_word_default_or() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = setup_test_index(&temp_dir);

        let searcher = Searcher::new(&index_path).unwrap();

        // 默认多词是 OR 搜索
        let results = searcher.search("error warning").unwrap();

        // 应该找到包含 error 或 warning 的文件
        assert!(!results.is_empty());
    }

    #[test]
    fn test_multi_word_and() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = setup_test_index(&temp_dir);

        let searcher = Searcher::new(&index_path).unwrap();

        // AND 搜索
        let results = searcher.search("+error +critical").unwrap();

        // 结果应该包含同时有 error 和 critical 的文件
        // error.log 包含 "ERROR: critical failure"
        if !results.is_empty() {
            assert!(results.iter().any(|r| r.path.to_string_lossy().contains("error.log")));
        }
    }
}

mod filter_search_tests {
    use super::*;

    #[test]
    fn test_search_with_file_type_filter() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = setup_test_index(&temp_dir);

        let searcher = Searcher::new(&index_path).unwrap();

        // 只搜索 .log 文件
        let results = searcher.search_with_filter("error", Some("log"), 100).unwrap();

        // 所有结果应该是 .log 文件
        for result in &results {
            let ext = result.path.extension().and_then(|e| e.to_str()).unwrap_or("");
            assert_eq!(ext, "log");
        }
    }

    #[test]
    fn test_search_with_txt_filter() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = setup_test_index(&temp_dir);

        let searcher = Searcher::new(&index_path).unwrap();

        // 只搜索 .txt 文件
        let results = searcher.search_with_filter("错误", Some("txt"), 100).unwrap();

        for result in &results {
            let ext = result.path.extension().and_then(|e| e.to_str()).unwrap_or("");
            assert_eq!(ext, "txt");
        }
    }

    #[test]
    fn test_search_with_no_filter() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = setup_test_index(&temp_dir);

        let searcher = Searcher::new(&index_path).unwrap();

        // 不带过滤器
        let results = searcher.search_with_filter("error", None, 100).unwrap();

        // 应该找到多种类型的文件
        let extensions: std::collections::HashSet<_> = results
            .iter()
            .filter_map(|r| r.path.extension().and_then(|e| e.to_str()))
            .collect();

        assert!(!extensions.is_empty());
    }
}

mod empty_result_tests {
    use super::*;

    #[test]
    fn test_search_no_results() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = setup_test_index(&temp_dir);

        let searcher = Searcher::new(&index_path).unwrap();
        let results = searcher.search("xyznonexistentterm12345xyz").unwrap();

        assert!(results.is_empty());
    }

    #[test]
    fn test_search_empty_query() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = setup_test_index(&temp_dir);

        let searcher = Searcher::new(&index_path).unwrap();

        // 空查询应该返回错误或空结果
        let results = searcher.search("");
        // Tantivy 可能会拒绝空查询
        assert!(results.is_err() || results.unwrap().is_empty());
    }

    #[test]
    fn test_search_whitespace_query() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = setup_test_index(&temp_dir);

        let searcher = Searcher::new(&index_path).unwrap();
        let results = searcher.search("   ");

        // 空白查询应该返回错误或空结果
        assert!(results.is_err() || results.unwrap().is_empty());
    }
}

mod score_ordering_tests {
    use super::*;

    #[test]
    fn test_results_sorted_by_score() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = setup_test_index(&temp_dir);

        let searcher = Searcher::new(&index_path).unwrap();
        let results = searcher.search("error").unwrap();

        // 验证结果按分数降序排列
        for i in 1..results.len() {
            assert!(
                results[i - 1].score >= results[i].score,
                "Results should be sorted by score in descending order"
            );
        }
    }

    #[test]
    fn test_multiple_matches_higher_score() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = setup_test_index(&temp_dir);

        let searcher = Searcher::new(&index_path).unwrap();
        let results = searcher.search("error").unwrap();

        // error.log 有多个 error 匹配，应该排名靠前
        if results.len() > 1 {
            // 第一个结果的分数应该最高
            let first_score = results[0].score;
            let last_score = results[results.len() - 1].score;
            assert!(first_score >= last_score);
        }
    }
}

mod snippet_tests {
    use super::*;

    #[test]
    fn test_snippet_generation() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = setup_test_index(&temp_dir);

        let searcher = Searcher::new(&index_path).unwrap();
        let results = searcher.search("error").unwrap();

        // 验证有片段生成
        for result in &results {
            assert!(result.snippet.is_some(), "Snippet should be generated");
        }
    }

    #[test]
    fn test_snippet_contains_highlight() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = setup_test_index(&temp_dir);

        let searcher = Searcher::new(&index_path).unwrap();
        let results = searcher.search("error").unwrap();

        // 验证片段包含高亮标记
        for result in &results {
            if let Some(ref snippet) = result.snippet {
                // 片段应该包含 ANSI 转义序列或 HTML 标记
                let has_highlight = snippet.contains("\x1b[") || snippet.contains("<b>");
                // 至少包含搜索词
                let contains_term = snippet.to_lowercase().contains("error");
                assert!(has_highlight || contains_term);
            }
        }
    }

    #[test]
    fn test_disable_highlight() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = setup_test_index(&temp_dir);

        let config = SearchConfig {
            enable_highlight: false,
            ..Default::default()
        };

        let searcher = Searcher::with_config(&index_path, config).unwrap();
        let results = searcher.search("error").unwrap();

        // 禁用高亮后，片段应该为 None
        for result in &results {
            assert!(result.snippet.is_none());
        }
    }
}

mod limit_tests {
    use super::*;

    #[test]
    fn test_search_with_limit() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = setup_test_index(&temp_dir);

        let searcher = Searcher::new(&index_path).unwrap();

        // 限制返回 2 个结果
        let results = searcher.search_with_limit("error", 2).unwrap();

        assert!(results.len() <= 2);
    }

    #[test]
    fn test_search_with_large_limit() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = setup_test_index(&temp_dir);

        let searcher = Searcher::new(&index_path).unwrap();

        // 限制返回 1000 个结果
        let results = searcher.search_with_limit("error", 1000).unwrap();

        // 结果数量不应超过索引中的文档数
        assert!(results.len() <= 7);
    }

    #[test]
    fn test_num_docs() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = setup_test_index(&temp_dir);

        let searcher = Searcher::new(&index_path).unwrap();

        // 验证文档数量
        assert_eq!(searcher.num_docs(), 7);
    }
}

mod special_character_tests {
    use super::*;

    #[test]
    fn test_search_with_special_chars() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = setup_test_index(&temp_dir);

        let searcher = Searcher::new(&index_path).unwrap();

        // 搜索包含特殊字符的内容
        let results = searcher.search("println").unwrap();

        assert!(!results.is_empty());
    }

    #[test]
    fn test_search_json_content() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = setup_test_index(&temp_dir);

        let searcher = Searcher::new(&index_path).unwrap();
        let results = searcher.search("name").unwrap();

        assert!(results.iter().any(|r| r.path.to_string_lossy().contains("data.json")));
    }
}
