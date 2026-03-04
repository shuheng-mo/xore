//! 分词器封装
//!
//! 提供统一的分词接口

use anyhow::Result;
use std::path::Path;
use tokenizers::{Encoding, Tokenizer as HFTokenizer};

/// 分词器
pub struct Tokenizer {
    tokenizer: HFTokenizer,
}

impl Tokenizer {
    /// 从文件加载分词器
    ///
    /// # 参数
    /// - `path`: tokenizer.json 文件路径
    ///
    /// # 返回
    /// 加载好的分词器
    pub fn load(path: &Path) -> Result<Self> {
        let tokenizer =
            HFTokenizer::from_file(path).map_err(|e| anyhow::anyhow!("加载分词器失败: {}", e))?;
        Ok(Self { tokenizer })
    }

    /// 分词并编码文本
    ///
    /// # 参数
    /// - `text`: 输入文本
    /// - `add_special_tokens`: 是否添加特殊 token（如 [CLS], [SEP]）
    ///
    /// # 返回
    /// 编码结果
    pub fn encode(&self, text: &str, add_special_tokens: bool) -> Result<Encoding> {
        self.tokenizer
            .encode(text, add_special_tokens)
            .map_err(|e| anyhow::anyhow!("编码失败: {}", e))
    }

    /// 批量编码
    ///
    /// # 参数
    /// - `texts`: 文本列表
    /// - `add_special_tokens`: 是否添加特殊 token
    ///
    /// # 返回
    /// 编码结果列表
    pub fn encode_batch(
        &self,
        texts: Vec<&str>,
        add_special_tokens: bool,
    ) -> Result<Vec<Encoding>> {
        self.tokenizer
            .encode_batch(texts, add_special_tokens)
            .map_err(|e| anyhow::anyhow!("批量编码失败: {}", e))
    }

    /// 解码 token IDs 为文本
    ///
    /// # 参数
    /// - `ids`: token ID 列表
    /// - `skip_special_tokens`: 是否跳过特殊 token
    ///
    /// # 返回
    /// 解码后的文本
    pub fn decode(&self, ids: &[u32], skip_special_tokens: bool) -> Result<String> {
        self.tokenizer
            .decode(ids, skip_special_tokens)
            .map_err(|e| anyhow::anyhow!("解码失败: {}", e))
    }

    /// 获取词汇表大小
    pub fn vocab_size(&self) -> usize {
        self.tokenizer.get_vocab_size(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 注意：这些测试需要实际的 tokenizer.json 文件才能运行
    // 在 CI 环境中可能需要跳过或使用 mock

    #[test]
    #[ignore] // 需要实际的 tokenizer 文件
    fn test_encode() {
        let tokenizer = Tokenizer::load(Path::new("assets/tokenizer.json")).unwrap();
        let encoding = tokenizer.encode("Hello, world!", true).unwrap();
        assert!(!encoding.get_ids().is_empty());
    }

    #[test]
    #[ignore]
    fn test_decode() {
        let tokenizer = Tokenizer::load(Path::new("assets/tokenizer.json")).unwrap();
        let encoding = tokenizer.encode("Hello, world!", true).unwrap();
        let decoded = tokenizer.decode(encoding.get_ids(), true).unwrap();
        assert!(decoded.contains("Hello"));
    }
}
