//! 类型定义模块单元测试

use std::path::PathBuf;
use xore_core::types::{DataQualityReport, FileType, SearchResult};

#[test]
fn test_search_result_creation() {
    let result = SearchResult {
        path: PathBuf::from("/tmp/test.txt"),
        line: Some(42),
        column: Some(10),
        score: 0.95,
        snippet: Some("test snippet".to_string()),
    };

    assert_eq!(result.path, PathBuf::from("/tmp/test.txt"));
    assert_eq!(result.line, Some(42));
    assert_eq!(result.column, Some(10));
    assert_eq!(result.score, 0.95);
    assert_eq!(result.snippet, Some("test snippet".to_string()));
}

#[test]
fn test_search_result_without_optional_fields() {
    let result = SearchResult {
        path: PathBuf::from("/tmp/file.txt"),
        line: None,
        column: None,
        score: 0.5,
        snippet: None,
    };

    assert!(result.line.is_none());
    assert!(result.column.is_none());
    assert!(result.snippet.is_none());
}

#[test]
fn test_search_result_clone() {
    let result = SearchResult {
        path: PathBuf::from("/tmp/test.txt"),
        line: Some(10),
        column: Some(5),
        score: 0.8,
        snippet: Some("snippet".to_string()),
    };

    let cloned = result.clone();
    assert_eq!(result.path, cloned.path);
    assert_eq!(result.line, cloned.line);
    assert_eq!(result.score, cloned.score);
}

#[test]
fn test_search_result_serialization() {
    let result = SearchResult {
        path: PathBuf::from("/tmp/test.txt"),
        line: Some(42),
        column: Some(10),
        score: 0.95,
        snippet: Some("test".to_string()),
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("test.txt"));
    assert!(json.contains("42"));
    assert!(json.contains("0.95"));
}

#[test]
fn test_search_result_deserialization() {
    let json = r#"{
        "path": "/tmp/test.txt",
        "line": 42,
        "column": 10,
        "score": 0.95,
        "snippet": "test snippet"
    }"#;

    let result: SearchResult = serde_json::from_str(json).unwrap();
    assert_eq!(result.line, Some(42));
    assert_eq!(result.column, Some(10));
    assert_eq!(result.score, 0.95);
}

#[test]
fn test_file_type_variants() {
    assert_eq!(FileType::Text, FileType::Text);
    assert_eq!(FileType::Csv, FileType::Csv);
    assert_eq!(FileType::Json, FileType::Json);
    assert_eq!(FileType::Log, FileType::Log);
    assert_eq!(FileType::Code, FileType::Code);
    assert_eq!(FileType::Binary, FileType::Binary);
    assert_eq!(FileType::Unknown, FileType::Unknown);
}

#[test]
fn test_file_type_equality() {
    assert_eq!(FileType::Text, FileType::Text);
    assert_ne!(FileType::Text, FileType::Csv);
    assert_ne!(FileType::Json, FileType::Log);
}

#[test]
fn test_file_type_clone() {
    let ft = FileType::Csv;
    let cloned = ft;
    assert_eq!(ft, cloned);
}

#[test]
fn test_file_type_serialization() {
    let ft = FileType::Csv;
    let json = serde_json::to_string(&ft).unwrap();
    assert!(json.contains("Csv"));
}

#[test]
fn test_file_type_deserialization() {
    let json = "\"Csv\"";
    let ft: FileType = serde_json::from_str(json).unwrap();
    assert_eq!(ft, FileType::Csv);
}

#[test]
fn test_data_quality_report_creation() {
    let report = DataQualityReport {
        row_count: 1000,
        column_count: 10,
        has_nulls: true,
        has_duplicates: false,
        has_outliers: true,
        suggestions: vec!["检查缺失值".to_string(), "移除离群值".to_string()],
    };

    assert_eq!(report.row_count, 1000);
    assert_eq!(report.column_count, 10);
    assert!(report.has_nulls);
    assert!(!report.has_duplicates);
    assert!(report.has_outliers);
    assert_eq!(report.suggestions.len(), 2);
}

#[test]
fn test_data_quality_report_no_issues() {
    let report = DataQualityReport {
        row_count: 100,
        column_count: 5,
        has_nulls: false,
        has_duplicates: false,
        has_outliers: false,
        suggestions: vec![],
    };

    assert!(!report.has_nulls);
    assert!(!report.has_duplicates);
    assert!(!report.has_outliers);
    assert!(report.suggestions.is_empty());
}

#[test]
fn test_data_quality_report_clone() {
    let report = DataQualityReport {
        row_count: 500,
        column_count: 8,
        has_nulls: true,
        has_duplicates: true,
        has_outliers: false,
        suggestions: vec!["建议1".to_string()],
    };

    let cloned = report.clone();
    assert_eq!(report.row_count, cloned.row_count);
    assert_eq!(report.column_count, cloned.column_count);
    assert_eq!(report.has_nulls, cloned.has_nulls);
    assert_eq!(report.suggestions.len(), cloned.suggestions.len());
}

#[test]
fn test_data_quality_report_serialization() {
    let report = DataQualityReport {
        row_count: 1000,
        column_count: 10,
        has_nulls: true,
        has_duplicates: false,
        has_outliers: true,
        suggestions: vec!["test".to_string()],
    };

    let json = serde_json::to_string(&report).unwrap();
    assert!(json.contains("1000"));
    assert!(json.contains("true"));
    assert!(json.contains("test"));
}

#[test]
fn test_data_quality_report_deserialization() {
    let json = r#"{
        "row_count": 1000,
        "column_count": 10,
        "has_nulls": true,
        "has_duplicates": false,
        "has_outliers": true,
        "suggestions": ["建议1", "建议2"]
    }"#;

    let report: DataQualityReport = serde_json::from_str(json).unwrap();
    assert_eq!(report.row_count, 1000);
    assert_eq!(report.column_count, 10);
    assert!(report.has_nulls);
    assert_eq!(report.suggestions.len(), 2);
}

#[test]
fn test_data_quality_report_debug_format() {
    let report = DataQualityReport {
        row_count: 100,
        column_count: 5,
        has_nulls: false,
        has_duplicates: false,
        has_outliers: false,
        suggestions: vec![],
    };

    let debug_str = format!("{:?}", report);
    assert!(debug_str.contains("DataQualityReport"));
    assert!(debug_str.contains("100"));
}
