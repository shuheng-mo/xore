//! XORE AI Module - 语义嵌入和向量搜索
//!
//! 这个crate提供基于ONNX的语义嵌入生成和向量相似度计算功能。

pub mod embedding;
pub mod tokenizer;

pub use embedding::EmbeddingModel;

#[cfg(test)]
mod tests {
    #[test]
    fn test_placeholder() {
        assert_eq!(2 + 2, 4);
    }
}
