//! XORE Data Processor - 数据处理引擎
//!
//! 这个crate提供基于Polars的高性能数据处理和SQL查询功能。

pub mod export;
pub mod parser;
pub mod profiler;
pub mod sql;

// 导出主要类型
pub use parser::{DataParser, ParserConfig};
pub use profiler::{
    ColumnStats, DataProfiler, MissingStats, OutlierInfo, QualityReport, Severity, Suggestion,
    SuggestionType,
};
pub use sql::SqlEngine;

// 重新导出 Polars 类型，方便使用
pub use polars::prelude::{AnyValue, DataFrame, LazyFrame};

#[cfg(test)]
mod tests {
    #[test]
    fn test_placeholder() {
        assert_eq!(2 + 2, 4);
    }
}
