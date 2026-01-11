//! XORE Data Processor - 数据处理引擎
//!
//! 这个crate提供基于Polars的高性能数据处理和SQL查询功能。

pub mod parser;
pub mod sql;
pub mod profiler;
pub mod export;

pub use parser::DataParser;
pub use sql::SqlEngine;
pub use profiler::DataProfiler;

use xore_core::Result;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_placeholder() {
        assert_eq!(2 + 2, 4);
    }
}
