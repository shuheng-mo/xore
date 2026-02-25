//! 索引构建器
//!
//! 基于 Tantivy 实现全文索引构建，支持：
//! - 中英文混合分词
//! - 增量索引（先删后增）
//! - 批量文档添加
//! - 大文件内存映射读取

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use anyhow::{Context, Result};
use memmap2::Mmap;
use tantivy::schema::{Field, Schema, STORED, STRING};
use tantivy::{doc, Index, IndexWriter, Term};
use tracing::{debug, info};

use crate::scanner::ScannedFile;
use crate::tokenizer::register_xore_tokenizer;

/// 索引 Schema 定义
///
/// 包含以下字段：
/// - path: 文件路径（唯一标识）
/// - content: 文件内容（全文索引）
/// - file_type: 文件类型（用于过滤）
/// - size: 文件大小
/// - modified: 修改时间
#[derive(Clone)]
pub struct IndexSchema {
    schema: Schema,
    path_field: Field,
    content_field: Field,
    file_type_field: Field,
    size_field: Field,
    modified_field: Field,
}

impl IndexSchema {
    /// 创建索引 Schema
    pub fn new() -> Self {
        let mut schema_builder = Schema::builder();

        // 文件路径：存储 + 精确匹配
        let path_field = schema_builder.add_text_field("path", STRING | STORED);

        // 文件内容：全文索引（使用自定义分词器）
        let text_options = tantivy::schema::TextOptions::default()
            .set_indexing_options(
                tantivy::schema::TextFieldIndexing::default()
                    .set_tokenizer("xore")
                    .set_index_option(tantivy::schema::IndexRecordOption::WithFreqsAndPositions),
            )
            .set_stored();
        let content_field = schema_builder.add_text_field("content", text_options);

        // 文件类型：精确匹配 + 存储
        let file_type_field = schema_builder.add_text_field("file_type", STRING | STORED);

        // 文件大小：数值索引
        let size_field = schema_builder.add_u64_field(
            "size",
            tantivy::schema::NumericOptions::default().set_indexed().set_stored(),
        );

        // 修改时间：数值索引（存储为 Unix 时间戳）
        let modified_field = schema_builder.add_u64_field(
            "modified",
            tantivy::schema::NumericOptions::default().set_indexed().set_stored(),
        );

        Self {
            schema: schema_builder.build(),
            path_field,
            content_field,
            file_type_field,
            size_field,
            modified_field,
        }
    }

    /// 获取 Tantivy Schema
    pub fn schema(&self) -> &Schema {
        &self.schema
    }

    /// 获取路径字段
    pub fn path_field(&self) -> Field {
        self.path_field
    }

    /// 获取内容字段
    pub fn content_field(&self) -> Field {
        self.content_field
    }

    /// 获取文件类型字段
    pub fn file_type_field(&self) -> Field {
        self.file_type_field
    }

    /// 获取大小字段
    pub fn size_field(&self) -> Field {
        self.size_field
    }

    /// 获取修改时间字段
    pub fn modified_field(&self) -> Field {
        self.modified_field
    }
}

impl Default for IndexSchema {
    fn default() -> Self {
        Self::new()
    }
}

/// 索引构建器配置
#[derive(Debug, Clone)]
pub struct IndexConfig {
    /// 索引目录路径
    pub index_path: PathBuf,
    /// Writer 缓冲区大小（字节）
    pub writer_buffer_size: usize,
    /// 单个文件最大大小（字节），超过则跳过
    pub max_file_size: u64,
    /// 是否使用内存映射读取大文件
    pub use_mmap: bool,
    /// 使用内存映射的文件大小阈值
    pub mmap_threshold: u64,
}

impl Default for IndexConfig {
    fn default() -> Self {
        Self {
            index_path: PathBuf::from(".xore/index"),
            writer_buffer_size: 50_000_000,   // 50MB
            max_file_size: 100 * 1024 * 1024, // 100MB
            use_mmap: true,
            mmap_threshold: 1024 * 1024, // 1MB
        }
    }
}

/// 索引构建器
///
/// 负责创建、更新和管理 Tantivy 索引。
pub struct IndexBuilder {
    index: Index,
    writer: IndexWriter,
    schema: IndexSchema,
    config: IndexConfig,
    documents_added: usize,
    documents_deleted: usize,
    errors: Vec<String>,
}

impl IndexBuilder {
    /// 创建新的索引构建器
    ///
    /// 如果索引目录不存在，会自动创建。
    /// 如果索引已存在，会打开现有索引。
    pub fn new(index_path: &Path) -> Result<Self> {
        Self::with_config(IndexConfig {
            index_path: index_path.to_path_buf(),
            ..Default::default()
        })
    }

    /// 使用自定义配置创建索引构建器
    pub fn with_config(config: IndexConfig) -> Result<Self> {
        let schema = IndexSchema::new();

        // 创建索引目录
        std::fs::create_dir_all(&config.index_path).with_context(|| {
            format!("Failed to create index directory: {:?}", config.index_path)
        })?;

        // 打开或创建索引
        let index = if config.index_path.join("meta.json").exists() {
            info!("Opening existing index at {:?}", config.index_path);
            Index::open_in_dir(&config.index_path)
                .with_context(|| format!("Failed to open index at {:?}", config.index_path))?
        } else {
            info!("Creating new index at {:?}", config.index_path);
            Index::create_in_dir(&config.index_path, schema.schema().clone())
                .with_context(|| format!("Failed to create index at {:?}", config.index_path))?
        };

        // 注册自定义分词器
        register_xore_tokenizer(&index)?;

        // 创建 IndexWriter
        let writer = index
            .writer(config.writer_buffer_size)
            .with_context(|| "Failed to create index writer")?;

        Ok(Self {
            index,
            writer,
            schema,
            config,
            documents_added: 0,
            documents_deleted: 0,
            errors: Vec::new(),
        })
    }

    /// 添加单个文档到索引
    pub fn add_document(&mut self, file: &ScannedFile) -> Result<()> {
        // 跳过目录
        if file.is_dir {
            return Ok(());
        }

        // 检查文件大小
        if file.size > self.config.max_file_size {
            debug!("Skipping large file: {:?} ({} bytes)", file.path, file.size);
            return Ok(());
        }

        // 读取文件内容
        let content = match self.read_file_content(&file.path, file.size) {
            Ok(c) => c,
            Err(e) => {
                self.errors.push(format!("{:?}: {}", file.path, e));
                return Ok(()); // 继续处理其他文件
            }
        };

        // 检查是否为二进制文件
        if is_binary_content(&content) {
            debug!("Skipping binary file: {:?}", file.path);
            return Ok(());
        }

        // 先删除可能存在的旧文档
        let path_str = file.path.to_string_lossy().to_string();
        let term = Term::from_field_text(self.schema.path_field(), &path_str);
        self.writer.delete_term(term);

        // 获取文件类型
        let file_type = detect_file_type(&file.path);

        // 获取修改时间戳
        let modified_ts = file
            .modified
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        // 添加文档
        self.writer.add_document(doc!(
            self.schema.path_field() => path_str,
            self.schema.content_field() => content,
            self.schema.file_type_field() => file_type,
            self.schema.size_field() => file.size,
            self.schema.modified_field() => modified_ts,
        ))?;

        self.documents_added += 1;

        Ok(())
    }

    /// 批量添加文档
    pub fn add_documents_batch(&mut self, files: &[ScannedFile]) -> Result<usize> {
        let mut added = 0;

        for file in files {
            if let Err(e) = self.add_document(file) {
                self.errors.push(format!("{:?}: {}", file.path, e));
            } else {
                added += 1;
            }
        }

        Ok(added)
    }

    /// 删除指定路径的文档
    pub fn delete_document(&mut self, path: &Path) -> Result<()> {
        let path_str = path.to_string_lossy().to_string();
        let term = Term::from_field_text(self.schema.path_field(), &path_str);
        self.writer.delete_term(term);
        self.documents_deleted += 1;
        Ok(())
    }

    /// 提交并优化索引
    pub fn build(mut self) -> Result<IndexStats> {
        info!("Committing index...");
        self.writer.commit().with_context(|| "Failed to commit index")?;

        // 等待合并完成
        self.writer.wait_merging_threads()?;

        let stats = IndexStats {
            documents_added: self.documents_added,
            documents_deleted: self.documents_deleted,
            errors: self.errors,
            index_path: self.config.index_path,
        };

        info!(
            "Index built successfully: {} documents added, {} deleted, {} errors",
            stats.documents_added,
            stats.documents_deleted,
            stats.errors.len()
        );

        Ok(stats)
    }

    /// 获取当前索引
    pub fn index(&self) -> &Index {
        &self.index
    }

    /// 获取 Schema
    pub fn schema(&self) -> &IndexSchema {
        &self.schema
    }

    /// 读取文件内容
    fn read_file_content(&self, path: &Path, size: u64) -> Result<String> {
        // 对于大文件使用内存映射
        if self.config.use_mmap && size > self.config.mmap_threshold {
            self.read_file_mmap(path)
        } else {
            self.read_file_direct(path)
        }
    }

    /// 使用内存映射读取文件
    fn read_file_mmap(&self, path: &Path) -> Result<String> {
        let file = File::open(path).with_context(|| format!("Failed to open file: {:?}", path))?;

        let mmap = unsafe { Mmap::map(&file) }
            .with_context(|| format!("Failed to mmap file: {:?}", path))?;

        // 尝试 UTF-8 解码
        match std::str::from_utf8(&mmap) {
            Ok(s) => Ok(s.to_string()),
            Err(_) => {
                // 尝试 lossy 转换
                Ok(String::from_utf8_lossy(&mmap).into_owned())
            }
        }
    }

    /// 直接读取文件
    fn read_file_direct(&self, path: &Path) -> Result<String> {
        let mut file =
            File::open(path).with_context(|| format!("Failed to open file: {:?}", path))?;

        let mut content = String::new();
        match file.read_to_string(&mut content) {
            Ok(_) => Ok(content),
            Err(_) => {
                // 尝试 lossy 读取
                let mut bytes = Vec::new();
                let mut file = File::open(path)?;
                file.read_to_end(&mut bytes)?;
                Ok(String::from_utf8_lossy(&bytes).into_owned())
            }
        }
    }
}

/// 索引构建统计信息
#[derive(Debug)]
pub struct IndexStats {
    /// 添加的文档数
    pub documents_added: usize,
    /// 删除的文档数
    pub documents_deleted: usize,
    /// 错误列表
    pub errors: Vec<String>,
    /// 索引路径
    pub index_path: PathBuf,
}

/// 检测内容是否为二进制
fn is_binary_content(content: &str) -> bool {
    // 检查前 8000 字节中是否有 null 字符
    let check_len = content.len().min(8000);
    content[..check_len].contains('\0')
}

/// 检测文件类型
fn detect_file_type(path: &Path) -> String {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_else(|| "unknown".to_string())
}

/// 打开已存在的索引（只读）
pub fn open_index(index_path: &Path) -> Result<(Index, IndexSchema)> {
    let schema = IndexSchema::new();
    let index = Index::open_in_dir(index_path)
        .with_context(|| format!("Failed to open index at {:?}", index_path))?;

    // 注册自定义分词器
    register_xore_tokenizer(&index)?;

    Ok((index, schema))
}

/// 检查索引是否存在
pub fn index_exists(index_path: &Path) -> bool {
    index_path.join("meta.json").exists()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_file(dir: &Path, name: &str, content: &str) -> PathBuf {
        let path = dir.join(name);
        let mut file = File::create(&path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        path
    }

    #[test]
    fn test_schema_creation() {
        let schema = IndexSchema::new();
        assert!(schema.schema().get_field("path").is_ok());
        assert!(schema.schema().get_field("content").is_ok());
        assert!(schema.schema().get_field("file_type").is_ok());
        assert!(schema.schema().get_field("size").is_ok());
        assert!(schema.schema().get_field("modified").is_ok());
    }

    #[test]
    fn test_index_builder_creation() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = temp_dir.path().join("test_index");

        let _builder = IndexBuilder::new(&index_path).unwrap();
        assert!(index_path.exists());
    }

    #[test]
    fn test_add_single_document() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = temp_dir.path().join("test_index");
        let files_dir = temp_dir.path().join("files");
        std::fs::create_dir_all(&files_dir).unwrap();

        // 创建测试文件
        let test_file_path = create_test_file(&files_dir, "test.txt", "Hello World 你好世界");

        let mut builder = IndexBuilder::new(&index_path).unwrap();

        let scanned_file = ScannedFile {
            path: test_file_path,
            size: 23,
            modified: Some(SystemTime::now()),
            is_dir: false,
        };

        builder.add_document(&scanned_file).unwrap();
        let stats = builder.build().unwrap();

        assert_eq!(stats.documents_added, 1);
        assert!(stats.errors.is_empty());
    }

    #[test]
    fn test_batch_add_documents() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = temp_dir.path().join("test_index");
        let files_dir = temp_dir.path().join("files");
        std::fs::create_dir_all(&files_dir).unwrap();

        let file1 = create_test_file(&files_dir, "test1.txt", "Content 1");
        let file2 = create_test_file(&files_dir, "test2.txt", "Content 2");

        let files = vec![
            ScannedFile { path: file1, size: 9, modified: Some(SystemTime::now()), is_dir: false },
            ScannedFile { path: file2, size: 9, modified: Some(SystemTime::now()), is_dir: false },
        ];

        let mut builder = IndexBuilder::new(&index_path).unwrap();
        let added = builder.add_documents_batch(&files).unwrap();
        let stats = builder.build().unwrap();

        assert_eq!(added, 2);
        assert_eq!(stats.documents_added, 2);
    }

    #[test]
    fn test_skip_binary_file() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = temp_dir.path().join("test_index");
        let files_dir = temp_dir.path().join("files");
        std::fs::create_dir_all(&files_dir).unwrap();

        // 创建二进制文件
        let binary_path = files_dir.join("binary.bin");
        let mut file = File::create(&binary_path).unwrap();
        file.write_all(&[0x00, 0x01, 0x02, 0x00]).unwrap();

        let mut builder = IndexBuilder::new(&index_path).unwrap();

        let scanned_file = ScannedFile {
            path: binary_path,
            size: 4,
            modified: Some(SystemTime::now()),
            is_dir: false,
        };

        builder.add_document(&scanned_file).unwrap();
        let stats = builder.build().unwrap();

        // 二进制文件应该被跳过
        assert_eq!(stats.documents_added, 0);
    }

    #[test]
    fn test_skip_directory() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = temp_dir.path().join("test_index");

        let mut builder = IndexBuilder::new(&index_path).unwrap();

        let scanned_file = ScannedFile {
            path: temp_dir.path().to_path_buf(),
            size: 0,
            modified: Some(SystemTime::now()),
            is_dir: true,
        };

        builder.add_document(&scanned_file).unwrap();
        let stats = builder.build().unwrap();

        assert_eq!(stats.documents_added, 0);
    }

    #[test]
    fn test_detect_file_type() {
        assert_eq!(detect_file_type(Path::new("test.rs")), "rs");
        assert_eq!(detect_file_type(Path::new("test.py")), "py");
        assert_eq!(detect_file_type(Path::new("test.TXT")), "txt");
        assert_eq!(detect_file_type(Path::new("noextension")), "unknown");
    }

    #[test]
    fn test_is_binary_content() {
        assert!(!is_binary_content("Hello World"));
        assert!(!is_binary_content("你好世界"));
        assert!(is_binary_content("Hello\0World"));
    }

    #[test]
    fn test_index_exists() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = temp_dir.path().join("test_index");

        assert!(!index_exists(&index_path));

        let builder = IndexBuilder::new(&index_path).unwrap();
        builder.build().unwrap();

        assert!(index_exists(&index_path));
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
            let scanned_file = ScannedFile {
                path: test_file_path.clone(),
                size: 16,
                modified: Some(SystemTime::now()),
                is_dir: false,
            };
            builder.add_document(&scanned_file).unwrap();
            builder.build().unwrap();
        }

        // 更新文件
        create_test_file(&files_dir, "test.txt", "Updated content");

        // 重新索引（应该自动删除旧文档）
        {
            let mut builder = IndexBuilder::new(&index_path).unwrap();
            let scanned_file = ScannedFile {
                path: test_file_path,
                size: 15,
                modified: Some(SystemTime::now()),
                is_dir: false,
            };
            builder.add_document(&scanned_file).unwrap();
            let stats = builder.build().unwrap();

            // 由于是更新，只计数一次
            assert_eq!(stats.documents_added, 1);
        }
    }
}
