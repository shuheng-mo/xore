//! XORE Search Engine - 全文搜索和语义搜索
//!
//! 这个crate提供基于Tantivy的全文搜索和基于ONNX的语义搜索功能。

pub mod indexer;
pub mod query;
pub mod watcher;

pub use indexer::IndexBuilder;
pub use query::Searcher;

use xore_core::Result;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_placeholder() {
        assert_eq!(2 + 2, 4);
    }
}
