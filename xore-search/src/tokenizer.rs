//! 中英文混合分词器
//!
//! 使用 jieba-rs 对中文进行分词，对英文使用标准分词。
//! 自动检测文本语言并选择合适的分词策略。

use jieba_rs::Jieba;
use std::sync::Arc;
use tantivy::tokenizer::{Token, TokenStream, Tokenizer};

/// XORE 中英文混合分词器
///
/// 自动识别中文和英文，对中文使用 jieba 分词，
/// 对英文使用空格和标点分割。
#[derive(Clone)]
pub struct XoreTokenizer {
    jieba: Arc<Jieba>,
}

impl XoreTokenizer {
    /// 创建新的分词器
    pub fn new() -> Self {
        Self { jieba: Arc::new(Jieba::new()) }
    }

    /// 使用自定义 jieba 实例创建分词器
    pub fn with_jieba(jieba: Arc<Jieba>) -> Self {
        Self { jieba }
    }
}

impl Default for XoreTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Tokenizer for XoreTokenizer {
    type TokenStream<'a> = XoreTokenStream<'a>;

    fn token_stream<'a>(&'a mut self, text: &'a str) -> Self::TokenStream<'a> {
        XoreTokenStream::new(text, self.jieba.clone())
    }
}

/// XORE Token 流
pub struct XoreTokenStream<'a> {
    #[allow(dead_code)]
    text: &'a str,
    #[allow(dead_code)]
    jieba: Arc<Jieba>,
    tokens: Vec<TokenInfo>,
    current_index: usize,
    token: Token,
}

struct TokenInfo {
    text: String,
    offset_from: usize,
    offset_to: usize,
}

impl<'a> XoreTokenStream<'a> {
    fn new(text: &'a str, jieba: Arc<Jieba>) -> Self {
        let tokens = Self::tokenize(text, &jieba);
        Self { text, jieba, tokens, current_index: 0, token: Token::default() }
    }

    /// 对文本进行分词，返回 token 列表
    fn tokenize(text: &str, jieba: &Jieba) -> Vec<TokenInfo> {
        let mut tokens = Vec::new();

        // 使用 jieba 的 cut_for_search 模式进行分词
        // 这个模式会把长词切成短词，有利于搜索召回
        for (start, end) in Self::segment_text(text, jieba) {
            let word = &text[start..end];
            let trimmed = word.trim();

            if !trimmed.is_empty() {
                // 转小写以便于搜索
                let lower = trimmed.to_lowercase();
                tokens.push(TokenInfo { text: lower, offset_from: start, offset_to: end });
            }
        }

        tokens
    }

    /// 分割文本，返回每个词的 (start, end) 位置
    fn segment_text(text: &str, jieba: &Jieba) -> Vec<(usize, usize)> {
        let mut segments = Vec::new();

        // 将文本按中英文分块处理
        let mut current_block_start = 0;
        let mut in_cjk = false;
        let chars_iter = text.char_indices().peekable();

        for (idx, ch) in chars_iter {
            let is_cjk = is_cjk_char(ch);

            if is_cjk != in_cjk && idx > current_block_start {
                // 语言切换，处理之前的块
                let block = &text[current_block_start..idx];
                if in_cjk {
                    // 中文块使用 jieba 分词
                    Self::segment_cjk(block, current_block_start, jieba, &mut segments);
                } else {
                    // 英文块使用空格分词
                    Self::segment_latin(block, current_block_start, &mut segments);
                }
                current_block_start = idx;
            }
            in_cjk = is_cjk;
        }

        // 处理最后一个块
        if current_block_start < text.len() {
            let block = &text[current_block_start..];
            if in_cjk {
                Self::segment_cjk(block, current_block_start, jieba, &mut segments);
            } else {
                Self::segment_latin(block, current_block_start, &mut segments);
            }
        }

        segments
    }

    /// 对中文文本进行分词
    fn segment_cjk(text: &str, base_offset: usize, jieba: &Jieba, segments: &mut Vec<(usize, usize)>) {
        // 使用 cut_for_search 获取更细粒度的分词结果
        let words = jieba.cut_for_search(text, true);

        let mut byte_offset = 0;
        for word in words {
            let word_start = text[byte_offset..].find(word);
            if let Some(start) = word_start {
                let absolute_start = base_offset + byte_offset + start;
                let absolute_end = absolute_start + word.len();
                segments.push((absolute_start, absolute_end));
                byte_offset += start + word.len();
            }
        }
    }

    /// 对英文文本进行分词（按空格和标点分割）
    fn segment_latin(text: &str, base_offset: usize, segments: &mut Vec<(usize, usize)>) {
        let mut word_start: Option<usize> = None;

        for (idx, ch) in text.char_indices() {
            if ch.is_alphanumeric() || ch == '_' {
                if word_start.is_none() {
                    word_start = Some(idx);
                }
            } else if let Some(start) = word_start {
                segments.push((base_offset + start, base_offset + idx));
                word_start = None;
            }
        }

        // 处理最后一个单词
        if let Some(start) = word_start {
            segments.push((base_offset + start, base_offset + text.len()));
        }
    }
}

impl<'a> TokenStream for XoreTokenStream<'a> {
    fn advance(&mut self) -> bool {
        if self.current_index < self.tokens.len() {
            let token_info = &self.tokens[self.current_index];
            self.token.text.clear();
            self.token.text.push_str(&token_info.text);
            self.token.offset_from = token_info.offset_from;
            self.token.offset_to = token_info.offset_to;
            self.token.position = self.current_index;
            self.token.position_length = 1;
            self.current_index += 1;
            true
        } else {
            false
        }
    }

    fn token(&self) -> &Token {
        &self.token
    }

    fn token_mut(&mut self) -> &mut Token {
        &mut self.token
    }
}

/// 判断字符是否为 CJK（中日韩）字符
fn is_cjk_char(ch: char) -> bool {
    matches!(ch,
        '\u{4E00}'..='\u{9FFF}' |     // CJK Unified Ideographs
        '\u{3400}'..='\u{4DBF}' |     // CJK Unified Ideographs Extension A
        '\u{20000}'..='\u{2A6DF}' |   // CJK Unified Ideographs Extension B
        '\u{2A700}'..='\u{2B73F}' |   // CJK Unified Ideographs Extension C
        '\u{2B740}'..='\u{2B81F}' |   // CJK Unified Ideographs Extension D
        '\u{2B820}'..='\u{2CEAF}' |   // CJK Unified Ideographs Extension E
        '\u{F900}'..='\u{FAFF}' |     // CJK Compatibility Ideographs
        '\u{2F800}'..='\u{2FA1F}'     // CJK Compatibility Ideographs Supplement
    )
}

/// 注册 XORE 分词器到 Tantivy 的 TokenizerManager
pub fn register_xore_tokenizer(
    index: &tantivy::Index,
) -> tantivy::Result<()> {
    index.tokenizers().register("xore", XoreTokenizer::new());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokenize_text(text: &str) -> Vec<String> {
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
        let tokens = tokenize_text("数据处理");
        assert!(tokens.contains(&"数据".to_string()));
        assert!(tokens.contains(&"处理".to_string()));
    }

    #[test]
    fn test_english_tokenization() {
        let tokens = tokenize_text("hello world");
        assert_eq!(tokens, vec!["hello", "world"]);
    }

    #[test]
    fn test_mixed_text() {
        let tokens = tokenize_text("error 错误处理 log");
        assert!(tokens.contains(&"error".to_string()));
        assert!(tokens.contains(&"错误".to_string()));
        assert!(tokens.contains(&"处理".to_string()));
        assert!(tokens.contains(&"log".to_string()));
    }

    #[test]
    fn test_case_insensitive() {
        let tokens = tokenize_text("Hello WORLD");
        assert_eq!(tokens, vec!["hello", "world"]);
    }

    #[test]
    fn test_empty_text() {
        let tokens = tokenize_text("");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_punctuation() {
        let tokens = tokenize_text("hello, world! 你好，世界！");
        assert!(tokens.contains(&"hello".to_string()));
        assert!(tokens.contains(&"world".to_string()));
        assert!(tokens.contains(&"你好".to_string()));
        assert!(tokens.contains(&"世界".to_string()));
    }

    #[test]
    fn test_numbers() {
        let tokens = tokenize_text("test123 error404");
        assert!(tokens.contains(&"test123".to_string()));
        assert!(tokens.contains(&"error404".to_string()));
    }

    #[test]
    fn test_cjk_detection() {
        assert!(is_cjk_char('中'));
        assert!(is_cjk_char('文'));
        assert!(!is_cjk_char('a'));
        assert!(!is_cjk_char('1'));
        assert!(!is_cjk_char(' '));
    }
}
