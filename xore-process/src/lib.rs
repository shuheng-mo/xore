//! XORE Data Processor - 数据处理引擎
//!
//! 这个crate提供基于Polars的高性能数据处理和SQL查询功能。

pub mod export;
pub mod parser;
pub mod profiler;
pub mod sql;

pub use parser::DataParser;
pub use profiler::DataProfiler;
pub use sql::SqlEngine;

#[cfg(test)]
mod tests {
    #[test]
    fn test_placeholder() {
        assert_eq!(2 + 2, 4);
    }
}
