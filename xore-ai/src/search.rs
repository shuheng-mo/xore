//! 向量搜索引擎
//!
//! 基于嵌入向量的语义搜索

use crate::EmbeddingModel;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{debug, info};

/// 文档
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// 文档 ID
    pub id: String,
    /// 文件路径
    pub path: PathBuf,
    /// 文档内容
    pub content: String,
}

/// 搜索结果
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// 文档
    pub document: Document,
    /// 相似度分数 [0, 1]
    pub score: f32,
}

/// 向量搜索引擎
pub struct VectorSearcher {
    /// 嵌入模型
    model: EmbeddingModel,
    /// 文档集合
    documents: Vec<Document>,
    /// 预计算的嵌入向量
    index: Vec<Vec<f32>>,
}

impl VectorSearcher {
    /// 创建新的向量搜索引擎
    ///
    /// # 参数
    /// - `model`: 嵌入模型
    ///
    /// # 返回
    /// 向量搜索引擎实例
    pub fn new(model: EmbeddingModel) -> Self {
        Self { model, documents: Vec::new(), index: Vec::new() }
    }

    /// 添加文档到索引
    ///
    /// # 参数
    /// - `doc`: 文档
    ///
    /// # 返回
    /// 成功或错误
    ///
    /// # 注意
    /// 需要 &mut self 因为 encode() 需要可变引用
    pub fn add_document(&mut self, doc: Document) -> Result<()> {
        info!("添加文档到索引: {:?}", doc.path);

        // 生成嵌入向量（需要 &mut self.model）
        let embedding = self.model.encode(&doc.content)?;

        self.documents.push(doc);
        self.index.push(embedding);

        debug!("当前索引文档数: {}", self.documents.len());
        Ok(())
    }

    /// 批量添加文档
    ///
    /// # 参数
    /// - `docs`: 文档列表
    ///
    /// # 返回
    /// 成功添加的文档数量
    pub fn add_documents(&mut self, docs: Vec<Document>) -> Result<usize> {
        let total = docs.len();
        info!("批量添加 {} 个文档到索引", total);

        let mut success_count = 0;
        for doc in docs {
            if self.add_document(doc).is_ok() {
                success_count += 1;
            }
        }

        info!("成功添加 {}/{} 个文档", success_count, total);
        Ok(success_count)
    }

    /// 语义搜索
    ///
    /// # 参数
    /// - `query`: 查询文本
    /// - `top_k`: 返回结果数量
    ///
    /// # 返回
    /// 搜索结果列表（按相似度降序）
    pub fn search(&mut self, query: &str, top_k: usize) -> Result<Vec<SearchResult>> {
        if self.documents.is_empty() {
            return Ok(Vec::new());
        }

        debug!("语义搜索: \"{}\"", query);

        // 1. 查询向量化
        let query_embedding = self.model.encode(query)?;

        // 2. 计算余弦相似度
        let mut scores: Vec<(usize, f32)> = Vec::new();
        for (i, doc_embedding) in self.index.iter().enumerate() {
            let score = EmbeddingModel::cosine_similarity(&query_embedding, doc_embedding);
            scores.push((i, score));
        }

        // 3. 排序取 top_k
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // 4. 返回结果
        let results: Vec<SearchResult> = scores
            .into_iter()
            .take(top_k)
            .map(|(i, score)| SearchResult { document: self.documents[i].clone(), score })
            .collect();

        debug!("找到 {} 个结果", results.len());
        Ok(results)
    }

    /// 获取索引中的文档数量
    pub fn document_count(&self) -> usize {
        self.documents.len()
    }

    /// 清空索引
    pub fn clear(&mut self) {
        self.documents.clear();
        self.index.clear();
    }
}

/// 计算余弦相似度（独立函数）
///
/// # 参数
/// - `a`: 向量 A
/// - `b`: 向量 B
///
/// # 返回
/// 余弦相似度 [-1, 1]
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a > 0.0 && norm_b > 0.0 {
        dot / (norm_a * norm_b)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 1e-6);

        let c = vec![1.0, 0.0, 0.0];
        let d = vec![0.0, 1.0, 0.0];
        assert!((cosine_similarity(&c, &d) - 0.0).abs() < 1e-6);

        let e = vec![1.0, 1.0];
        let f = vec![-1.0, -1.0];
        assert!((cosine_similarity(&e, &f) + 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_edge_cases() {
        // 空向量
        let empty: Vec<f32> = vec![];
        assert_eq!(cosine_similarity(&empty, &empty), 0.0);

        // 长度不匹配
        let a = vec![1.0, 2.0];
        let b = vec![1.0, 2.0, 3.0];
        assert_eq!(cosine_similarity(&a, &b), 0.0);

        // 零向量
        let zero = vec![0.0, 0.0, 0.0];
        let nonzero = vec![1.0, 2.0, 3.0];
        assert_eq!(cosine_similarity(&zero, &nonzero), 0.0);
    }
}
