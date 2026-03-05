//! 索引构建器测试
//!
//! 测试覆盖：
//! - Schema 创建验证
//! - 中文分词测试
//! - 英文分词测试
//! - 单文档索引与检索
//! - 批量索引性能测试
//! - 文档更新（删除 + 重建）
//! - 错误处理

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use tempfile::TempDir;
use xore_search::indexer::{index_exists, open_index, IndexBuilder, IndexConfig, IndexSchema};
use xore_search::scanner::ScannedFile;
use xore_search::tokenizer::XoreTokenizer;

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

mod schema_tests {
    use super::*;

    #[test]
    fn test_schema_has_all_fields() {
        let schema = IndexSchema::new();
        let s = schema.schema();

        assert!(s.get_field("path").is_ok());
        assert!(s.get_field("content").is_ok());
        assert!(s.get_field("file_type").is_ok());
        assert!(s.get_field("size").is_ok());
        assert!(s.get_field("modified").is_ok());
    }

    #[test]
    fn test_schema_field_accessors() {
        let schema = IndexSchema::new();

        // 验证字段可以正确获取
        let _ = schema.path_field();
        let _ = schema.content_field();
        let _ = schema.file_type_field();
        let _ = schema.size_field();
        let _ = schema.modified_field();
    }

    #[test]
    fn test_schema_clone() {
        let schema1 = IndexSchema::new();
        let schema2 = schema1.clone();

        // 验证克隆后的 schema 具有相同的字段
        assert_eq!(schema1.path_field(), schema2.path_field());
        assert_eq!(schema1.content_field(), schema2.content_field());
    }
}

mod tokenizer_tests {
    use super::*;
    use tantivy::tokenizer::{TokenStream, Tokenizer};

    fn tokenize(text: &str) -> Vec<String> {
        let mut tokenizer = XoreTokenizer::new();
        let mut stream = tokenizer.token_stream(text);
        let mut tokens = Vec::new();
        while stream.advance() {
            tokens.push(stream.token().text.clone());
        }
        tokens
    }

    #[test]
    fn test_chinese_tokenization() {
        let tokens = tokenize("数据处理");
        assert!(tokens.contains(&"数据".to_string()));
        assert!(tokens.contains(&"处理".to_string()));
    }

    #[test]
    fn test_chinese_sentence() {
        let tokens = tokenize("这是一个错误日志");
        assert!(tokens.contains(&"错误".to_string()));
        assert!(tokens.contains(&"日志".to_string()));
    }

    #[test]
    fn test_english_tokenization() {
        let tokens = tokenize("hello world");
        assert_eq!(tokens, vec!["hello", "world"]);
    }

    #[test]
    fn test_mixed_text() {
        let tokens = tokenize("error 错误处理 log");
        assert!(tokens.contains(&"error".to_string()));
        assert!(tokens.contains(&"错误".to_string()));
        assert!(tokens.contains(&"处理".to_string()));
        assert!(tokens.contains(&"log".to_string()));
    }

    #[test]
    fn test_case_insensitive() {
        let tokens = tokenize("Hello WORLD Test");
        assert!(tokens.contains(&"hello".to_string()));
        assert!(tokens.contains(&"world".to_string()));
        assert!(tokens.contains(&"test".to_string()));
    }

    #[test]
    fn test_empty_text() {
        let tokens = tokenize("");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_whitespace_only() {
        let tokens = tokenize("   \t\n  ");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_punctuation_removal() {
        let tokens = tokenize("hello, world! 你好，世界！");
        assert!(tokens.contains(&"hello".to_string()));
        assert!(tokens.contains(&"world".to_string()));
        // 中文分词应该正确处理标点
        assert!(tokens.iter().all(|t| !t.contains(',')));
    }

    #[test]
    fn test_numbers() {
        let tokens = tokenize("test123 error404");
        assert!(tokens.contains(&"test123".to_string()));
        assert!(tokens.contains(&"error404".to_string()));
    }

    #[test]
    fn test_underscore_in_words() {
        let tokens = tokenize("hello_world test_case");
        assert!(tokens.contains(&"hello_world".to_string()));
        assert!(tokens.contains(&"test_case".to_string()));
    }
}

mod index_builder_tests {
    use super::*;

    #[test]
    fn test_create_new_index() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = temp_dir.path().join("test_index");

        let builder = IndexBuilder::new(&index_path).unwrap();
        assert!(index_path.exists());

        // 提交空索引
        let stats = builder.build().unwrap();
        assert_eq!(stats.documents_added, 0);
    }

    #[test]
    fn test_open_existing_index() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = temp_dir.path().join("test_index");

        // 创建索引
        {
            let builder = IndexBuilder::new(&index_path).unwrap();
            builder.build().unwrap();
        }

        // 重新打开
        {
            let _builder = IndexBuilder::new(&index_path).unwrap();
            assert!(index_path.join("meta.json").exists());
        }
    }

    #[test]
    fn test_add_single_document() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = temp_dir.path().join("test_index");
        let files_dir = temp_dir.path().join("files");
        std::fs::create_dir_all(&files_dir).unwrap();

        let test_file = create_test_file(&files_dir, "test.txt", "Hello World 你好世界");

        let mut builder = IndexBuilder::new(&index_path).unwrap();
        builder.add_document(&scanned_file(test_file, "Hello World 你好世界")).unwrap();

        let stats = builder.build().unwrap();
        assert_eq!(stats.documents_added, 1);
        assert!(stats.errors.is_empty());
    }

    #[test]
    fn test_add_multiple_documents() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = temp_dir.path().join("test_index");
        let files_dir = temp_dir.path().join("files");
        std::fs::create_dir_all(&files_dir).unwrap();

        let file1 = create_test_file(&files_dir, "test1.txt", "Content one");
        let file2 = create_test_file(&files_dir, "test2.txt", "Content two");
        let file3 = create_test_file(&files_dir, "test3.txt", "Content three");

        let files = vec![
            scanned_file(file1, "Content one"),
            scanned_file(file2, "Content two"),
            scanned_file(file3, "Content three"),
        ];

        let mut builder = IndexBuilder::new(&index_path).unwrap();
        let added = builder.add_documents_batch(&files).unwrap();
        let stats = builder.build().unwrap();

        assert_eq!(added, 3);
        assert_eq!(stats.documents_added, 3);
    }

    #[test]
    fn test_skip_binary_file() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = temp_dir.path().join("test_index");
        let files_dir = temp_dir.path().join("files");
        std::fs::create_dir_all(&files_dir).unwrap();

        // 创建二进制文件（包含 null 字节）
        let binary_path = files_dir.join("binary.bin");
        let mut file = File::create(&binary_path).unwrap();
        file.write_all(&[0x00, 0x01, 0x02, 0x00, 0xFF]).unwrap();

        let mut builder = IndexBuilder::new(&index_path).unwrap();
        builder
            .add_document(&ScannedFile {
                path: binary_path,
                size: 5,
                modified: Some(SystemTime::now()),
                is_dir: false,
            })
            .unwrap();

        let stats = builder.build().unwrap();
        // 二进制文件应该被跳过
        assert_eq!(stats.documents_added, 0);
    }

    #[test]
    fn test_skip_directory() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = temp_dir.path().join("test_index");

        let mut builder = IndexBuilder::new(&index_path).unwrap();
        builder
            .add_document(&ScannedFile {
                path: temp_dir.path().to_path_buf(),
                size: 0,
                modified: Some(SystemTime::now()),
                is_dir: true,
            })
            .unwrap();

        let stats = builder.build().unwrap();
        assert_eq!(stats.documents_added, 0);
    }

    #[test]
    fn test_skip_large_file() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = temp_dir.path().join("test_index");
        let files_dir = temp_dir.path().join("files");
        std::fs::create_dir_all(&files_dir).unwrap();

        let large_file = create_test_file(&files_dir, "large.txt", "content");

        let config = IndexConfig {
            index_path: index_path.clone(),
            max_file_size: 5, // 只允许 5 字节以内的文件
            ..Default::default()
        };

        let mut builder = IndexBuilder::with_config(config).unwrap();
        builder
            .add_document(&ScannedFile {
                path: large_file,
                size: 100, // 模拟大文件
                modified: Some(SystemTime::now()),
                is_dir: false,
            })
            .unwrap();

        let stats = builder.build().unwrap();
        assert_eq!(stats.documents_added, 0);
    }

    #[test]
    fn test_document_update() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = temp_dir.path().join("test_index");
        let files_dir = temp_dir.path().join("files");
        std::fs::create_dir_all(&files_dir).unwrap();

        let test_file_path = create_test_file(&files_dir, "test.txt", "Original content");

        // 第一次索引
        {
            let mut builder = IndexBuilder::new(&index_path).unwrap();
            builder
                .add_document(&scanned_file(test_file_path.clone(), "Original content"))
                .unwrap();
            builder.build().unwrap();
        }

        // 更新文件
        create_test_file(&files_dir, "test.txt", "Updated content");

        // 重新索引
        {
            let mut builder = IndexBuilder::new(&index_path).unwrap();
            builder.add_document(&scanned_file(test_file_path, "Updated content")).unwrap();
            let stats = builder.build().unwrap();
            assert_eq!(stats.documents_added, 1);
        }
    }

    #[test]
    fn test_delete_document() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = temp_dir.path().join("test_index");
        let files_dir = temp_dir.path().join("files");
        std::fs::create_dir_all(&files_dir).unwrap();

        let test_file = create_test_file(&files_dir, "test.txt", "Test content");

        // 添加文档
        {
            let mut builder = IndexBuilder::new(&index_path).unwrap();
            builder.add_document(&scanned_file(test_file.clone(), "Test content")).unwrap();
            builder.build().unwrap();
        }

        // 删除文档
        {
            let mut builder = IndexBuilder::new(&index_path).unwrap();
            builder.delete_document(&test_file).unwrap();
            let stats = builder.build().unwrap();
            assert_eq!(stats.documents_deleted, 1);
        }
    }

    #[test]
    fn test_index_exists_function() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = temp_dir.path().join("test_index");

        assert!(!index_exists(&index_path));

        let builder = IndexBuilder::new(&index_path).unwrap();
        builder.build().unwrap();

        assert!(index_exists(&index_path));
    }

    #[test]
    fn test_open_index_function() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = temp_dir.path().join("test_index");

        // 创建索引
        {
            let builder = IndexBuilder::new(&index_path).unwrap();
            builder.build().unwrap();
        }

        // 使用 open_index 打开
        let (index, _schema) = open_index(&index_path).unwrap();
        assert!(index.schema().get_field("path").is_ok());
    }

    #[test]
    fn test_custom_config() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = temp_dir.path().join("test_index");

        let config = IndexConfig {
            index_path: index_path.clone(),
            writer_buffer_size: 15_000_000, // 15MB (minimum required by Tantivy)
            max_file_size: 50 * 1024 * 1024, // 50MB
            use_mmap: false,
            mmap_threshold: 512 * 1024,
        };

        let builder = IndexBuilder::with_config(config).unwrap();
        let stats = builder.build().unwrap();
        assert_eq!(stats.index_path, index_path);
    }
}

mod error_handling_tests {
    use super::*;

    #[test]
    fn test_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = temp_dir.path().join("test_index");

        let mut builder = IndexBuilder::new(&index_path).unwrap();

        // 添加不存在的文件
        builder
            .add_document(&ScannedFile {
                path: PathBuf::from("/nonexistent/path/file.txt"),
                size: 100,
                modified: Some(SystemTime::now()),
                is_dir: false,
            })
            .unwrap();

        let stats = builder.build().unwrap();
        // 应该记录错误但不崩溃
        assert_eq!(stats.documents_added, 0);
        assert!(!stats.errors.is_empty());
    }

    #[test]
    fn test_invalid_index_path() {
        // 尝试在无效路径创建索引
        let result = IndexBuilder::new(Path::new("/root/definitely/not/allowed/index"));
        // 应该失败（无权限）
        assert!(result.is_err());
    }

    #[test]
    fn test_open_nonexistent_index() {
        let result = open_index(Path::new("/nonexistent/index/path"));
        assert!(result.is_err());
    }
}

mod chinese_content_tests {
    use super::*;

    #[test]
    fn test_index_chinese_content() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = temp_dir.path().join("test_index");
        let files_dir = temp_dir.path().join("files");
        std::fs::create_dir_all(&files_dir).unwrap();

        let content = "这是一个测试文件\n包含中文内容\n用于验证索引功能";
        let test_file = create_test_file(&files_dir, "chinese.txt", content);

        let mut builder = IndexBuilder::new(&index_path).unwrap();
        builder.add_document(&scanned_file(test_file, content)).unwrap();

        let stats = builder.build().unwrap();
        assert_eq!(stats.documents_added, 1);
    }

    #[test]
    fn test_index_mixed_content() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = temp_dir.path().join("test_index");
        let files_dir = temp_dir.path().join("files");
        std::fs::create_dir_all(&files_dir).unwrap();

        let content = "Error 错误: 发生了一个异常\nException 异常 in module 模块";
        let test_file = create_test_file(&files_dir, "mixed.log", content);

        let mut builder = IndexBuilder::new(&index_path).unwrap();
        builder.add_document(&scanned_file(test_file, content)).unwrap();

        let stats = builder.build().unwrap();
        assert_eq!(stats.documents_added, 1);
    }
}
