//! 查询引擎
//!
//! 基于 Tantivy 实现全文搜索，支持：
//! - BM25 排序
//! - 中英文混合查询
//! - 结果高亮
//! - 文件类型过滤

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use tantivy::collector::TopDocs;
use tantivy::query::{BooleanQuery, Occur, QueryParser, TermQuery};
use tantivy::schema::{IndexRecordOption, Value};
use tantivy::snippet::{Snippet, SnippetGenerator};
use tantivy::{Index, ReloadPolicy, Searcher as TantivySearcher, Term};
use tracing::{debug, info};

use crate::indexer::{open_index, IndexSchema};
use xore_core::types::SearchResult;

/// 搜索配置
#[derive(Debug, Clone)]
pub struct SearchConfig {
    /// 最大返回结果数
    pub limit: usize,
    /// 高亮片段最大长度
    pub snippet_max_length: usize,
    /// 是否启用高亮
    pub enable_highlight: bool,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self { limit: 100, snippet_max_length: 200, enable_highlight: true }
    }
}

/// 搜索器
///
/// 提供全文搜索功能，支持 BM25 排序和结果高亮。
pub struct Searcher {
    index: Index,
    schema: IndexSchema,
    reader: tantivy::IndexReader,
    config: SearchConfig,
}

impl Searcher {
    /// 创建新的搜索器
    pub fn new(index_path: &Path) -> Result<Self> {
        Self::with_config(index_path, SearchConfig::default())
    }

    /// 使用自定义配置创建搜索器
    pub fn with_config(index_path: &Path, config: SearchConfig) -> Result<Self> {
        let (index, schema) = open_index(index_path)?;

        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()
            .with_context(|| "Failed to create index reader")?;

        Ok(Self { index, schema, reader, config })
    }

    /// 执行搜索
    pub fn search(&self, query_str: &str) -> Result<Vec<SearchResult>> {
        self.search_with_limit(query_str, self.config.limit)
    }

    /// 执行搜索，指定结果数量
    pub fn search_with_limit(&self, query_str: &str, limit: usize) -> Result<Vec<SearchResult>> {
        info!("Searching for: {}", query_str);

        let searcher = self.reader.searcher();

        // 创建查询解析器，针对 content 字段
        let query_parser = QueryParser::for_index(&self.index, vec![self.schema.content_field()]);

        // 解析查询
        let query = query_parser
            .parse_query(query_str)
            .with_context(|| format!("Failed to parse query: {}", query_str))?;

        // 执行搜索
        let top_docs = searcher
            .search(&query, &TopDocs::with_limit(limit))
            .with_context(|| "Search failed")?;

        debug!("Found {} results", top_docs.len());

        // 创建高亮生成器
        let snippet_generator = if self.config.enable_highlight {
            Some(SnippetGenerator::create(&searcher, &query, self.schema.content_field())?)
        } else {
            None
        };

        // 转换结果
        let mut results = Vec::new();
        for (score, doc_address) in top_docs {
            if let Ok(doc) = searcher.doc(doc_address) {
                let result = self.doc_to_search_result(&doc, score, snippet_generator.as_ref(), &searcher)?;
                results.push(result);
            }
        }

        Ok(results)
    }

    /// 带文件类型过滤的搜索
    pub fn search_with_filter(
        &self,
        query_str: &str,
        file_type: Option<&str>,
        limit: usize,
    ) -> Result<Vec<SearchResult>> {
        info!("Searching for: {} with filter: {:?}", query_str, file_type);

        let searcher = self.reader.searcher();

        // 创建查询解析器
        let query_parser = QueryParser::for_index(&self.index, vec![self.schema.content_field()]);

        // 解析内容查询
        let content_query = query_parser
            .parse_query(query_str)
            .with_context(|| format!("Failed to parse query: {}", query_str))?;

        // 如果有文件类型过滤，创建组合查询
        let final_query: Box<dyn tantivy::query::Query> = if let Some(ft) = file_type {
            let type_term = Term::from_field_text(self.schema.file_type_field(), ft);
            let type_query = TermQuery::new(type_term, IndexRecordOption::Basic);

            Box::new(BooleanQuery::new(vec![
                (Occur::Must, content_query),
                (Occur::Must, Box::new(type_query)),
            ]))
        } else {
            content_query
        };

        // 执行搜索
        let top_docs = searcher
            .search(&*final_query, &TopDocs::with_limit(limit))
            .with_context(|| "Search failed")?;

        // 创建高亮生成器
        let snippet_generator = if self.config.enable_highlight {
            // 使用原始内容查询生成高亮
            let content_query = query_parser.parse_query(query_str)?;
            Some(SnippetGenerator::create(&searcher, &content_query, self.schema.content_field())?)
        } else {
            None
        };

        // 转换结果
        let mut results = Vec::new();
        for (score, doc_address) in top_docs {
            if let Ok(doc) = searcher.doc(doc_address) {
                let result = self.doc_to_search_result(&doc, score, snippet_generator.as_ref(), &searcher)?;
                results.push(result);
            }
        }

        Ok(results)
    }

    /// 将 Tantivy 文档转换为 SearchResult
    fn doc_to_search_result(
        &self,
        doc: &tantivy::TantivyDocument,
        score: f32,
        snippet_generator: Option<&SnippetGenerator>,
        _searcher: &TantivySearcher,
    ) -> Result<SearchResult> {
        // 获取路径
        let path = doc
            .get_first(self.schema.path_field())
            .and_then(|v| v.as_str())
            .map(PathBuf::from)
            .unwrap_or_default();

        // 获取内容用于生成片段
        let content = doc
            .get_first(self.schema.content_field())
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // 生成高亮片段
        let snippet = if let Some(generator) = snippet_generator {
            let snippet = generator.snippet(content);
            Some(self.format_snippet(&snippet))
        } else {
            None
        };

        // 尝试从片段中提取行号
        let (line, column) = self.extract_line_info(content, &snippet);

        Ok(SearchResult { path, line, column, score, snippet })
    }

    /// 格式化高亮片段
    fn format_snippet(&self, snippet: &Snippet) -> String {
        // 使用 ANSI 转义序列进行高亮
        snippet.to_html()
            .replace("<b>", "\x1b[1;33m")  // 黄色粗体
            .replace("</b>", "\x1b[0m")     // 重置
    }

    /// 从内容和片段中提取行号信息
    fn extract_line_info(&self, content: &str, snippet: &Option<String>) -> (Option<usize>, Option<usize>) {
        if let Some(ref snip) = snippet {
            // 去除 ANSI 转义序列以获取纯文本
            let clean_snippet = snip
                .replace("\x1b[1;33m", "")
                .replace("\x1b[0m", "");

            // 在内容中查找片段位置
            if let Some(pos) = content.find(clean_snippet.trim()) {
                let line_number = content[..pos].matches('\n').count() + 1;
                let line_start = content[..pos].rfind('\n').map(|p| p + 1).unwrap_or(0);
                let column = pos - line_start + 1;
                return (Some(line_number), Some(column));
            }
        }
        (None, None)
    }

    /// 获取索引中的文档数量
    pub fn num_docs(&self) -> u64 {
        self.reader.searcher().num_docs()
    }

    /// 获取 Schema
    pub fn schema(&self) -> &IndexSchema {
        &self.schema
    }
}

impl Default for Searcher {
    fn default() -> Self {
        // 使用默认路径，如果失败则 panic
        Self::new(Path::new(".xore/index")).expect("Failed to create default searcher")
    }
}

/// 搜索结果迭代器
pub struct SearchResultIter<'a> {
    searcher: &'a Searcher,
    query: String,
    offset: usize,
    batch_size: usize,
    current_batch: Vec<SearchResult>,
    current_index: usize,
    exhausted: bool,
}

impl<'a> SearchResultIter<'a> {
    /// 创建新的搜索结果迭代器
    pub fn new(searcher: &'a Searcher, query: &str, batch_size: usize) -> Self {
        Self {
            searcher,
            query: query.to_string(),
            offset: 0,
            batch_size,
            current_batch: Vec::new(),
            current_index: 0,
            exhausted: false,
        }
    }
}

impl<'a> Iterator for SearchResultIter<'a> {
    type Item = SearchResult;

    fn next(&mut self) -> Option<Self::Item> {
        if self.exhausted {
            return None;
        }

        // 如果当前批次已用完，加载下一批
        if self.current_index >= self.current_batch.len() {
            match self.searcher.search_with_limit(&self.query, self.offset + self.batch_size) {
                Ok(results) => {
                    if results.len() <= self.offset {
                        self.exhausted = true;
                        return None;
                    }
                    self.current_batch = results.into_iter().skip(self.offset).collect();
                    self.offset += self.current_batch.len();
                    self.current_index = 0;

                    if self.current_batch.is_empty() {
                        self.exhausted = true;
                        return None;
                    }
                }
                Err(_) => {
                    self.exhausted = true;
                    return None;
                }
            }
        }

        let result = self.current_batch.get(self.current_index).cloned();
        self.current_index += 1;
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indexer::IndexBuilder;
    use crate::scanner::ScannedFile;
    use std::fs::File;
    use std::io::Write;
    use std::time::SystemTime;
    use tempfile::TempDir;

    fn setup_test_index(temp_dir: &TempDir) -> PathBuf {
        let index_path = temp_dir.path().join("test_index");
        let files_dir = temp_dir.path().join("files");
        std::fs::create_dir_all(&files_dir).unwrap();

        // 创建测试文件
        let files = vec![
            ("error.log", "This is an error message\nAnother line with error\n"),
            ("chinese.txt", "这是一个错误日志\n数据处理完成\n"),
            ("mixed.txt", "Error 错误 processing data 数据处理\n"),
            ("hello.rs", "fn main() {\n    println!(\"Hello, world!\");\n}\n"),
        ];

        let mut scanned_files = Vec::new();
        for (name, content) in files {
            let path = files_dir.join(name);
            let mut file = File::create(&path).unwrap();
            file.write_all(content.as_bytes()).unwrap();
            scanned_files.push(ScannedFile {
                path,
                size: content.len() as u64,
                modified: Some(SystemTime::now()),
                is_dir: false,
            });
        }

        // 构建索引
        let mut builder = IndexBuilder::new(&index_path).unwrap();
        builder.add_documents_batch(&scanned_files).unwrap();
        builder.build().unwrap();

        index_path
    }

    #[test]
    fn test_search_english() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = setup_test_index(&temp_dir);

        let searcher = Searcher::new(&index_path).unwrap();
        let results = searcher.search("error").unwrap();

        assert!(!results.is_empty());
        // 应该找到 error.log 和 mixed.txt
        assert!(results.iter().any(|r| r.path.to_string_lossy().contains("error.log")));
    }

    #[test]
    fn test_search_chinese() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = setup_test_index(&temp_dir);

        let searcher = Searcher::new(&index_path).unwrap();
        let results = searcher.search("错误").unwrap();

        assert!(!results.is_empty());
        // 应该找到 chinese.txt 和 mixed.txt
        assert!(results.iter().any(|r| r.path.to_string_lossy().contains("chinese.txt")));
    }

    #[test]
    fn test_search_mixed() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = setup_test_index(&temp_dir);

        let searcher = Searcher::new(&index_path).unwrap();
        let results = searcher.search("数据").unwrap();

        assert!(!results.is_empty());
    }

    #[test]
    fn test_search_with_filter() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = setup_test_index(&temp_dir);

        let searcher = Searcher::new(&index_path).unwrap();

        // 搜索 .log 文件
        let results = searcher.search_with_filter("error", Some("log"), 100).unwrap();

        // 应该只找到 error.log
        assert!(results.iter().all(|r| r.path.extension().map(|e| e == "log").unwrap_or(false)));
    }

    #[test]
    fn test_search_no_results() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = setup_test_index(&temp_dir);

        let searcher = Searcher::new(&index_path).unwrap();
        let results = searcher.search("nonexistentterm12345").unwrap();

        assert!(results.is_empty());
    }

    #[test]
    fn test_search_score_ordering() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = setup_test_index(&temp_dir);

        let searcher = Searcher::new(&index_path).unwrap();
        let results = searcher.search("error").unwrap();

        // 验证结果按分数降序排列
        for i in 1..results.len() {
            assert!(results[i - 1].score >= results[i].score);
        }
    }

    #[test]
    fn test_num_docs() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = setup_test_index(&temp_dir);

        let searcher = Searcher::new(&index_path).unwrap();
        assert_eq!(searcher.num_docs(), 4);
    }

    #[test]
    fn test_snippet_generation() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = setup_test_index(&temp_dir);

        let searcher = Searcher::new(&index_path).unwrap();
        let results = searcher.search("error").unwrap();

        // 验证片段不为空
        for result in &results {
            assert!(result.snippet.is_some());
        }
    }

    #[test]
    fn test_phrase_search() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = setup_test_index(&temp_dir);

        let searcher = Searcher::new(&index_path).unwrap();

        // 短语搜索
        let results = searcher.search("\"Hello, world\"").unwrap();

        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.path.to_string_lossy().contains("hello.rs")));
    }
}
