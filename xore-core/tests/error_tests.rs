//! 错误处理模块单元测试

use xore_core::{
    ErrorChain, ErrorContext, ErrorFormatter, ErrorFormatterConfig, ErrorHint, Result, XoreError,
    XoreErrorExt,
};

#[test]
fn test_index_error() {
    let err = XoreError::IndexError("索引构建失败".to_string());
    assert_eq!(err.to_string(), "索引错误: 索引构建失败");
}

#[test]
fn test_process_error() {
    let err = XoreError::ProcessError("解析CSV失败".to_string());
    assert_eq!(err.to_string(), "数据处理错误: 解析CSV失败");
}

#[test]
fn test_sql_error() {
    let err = XoreError::SqlError("语法错误".to_string());
    assert_eq!(err.to_string(), "SQL语法错误: 语法错误");
}

#[test]
fn test_file_not_found() {
    let err = XoreError::FileNotFound { path: "/path/to/file.txt".to_string() };
    assert_eq!(err.to_string(), "文件不存在: /path/to/file.txt");
}

#[test]
fn test_config_error() {
    let err = XoreError::ConfigError("配置文件格式错误".to_string());
    assert_eq!(err.to_string(), "配置错误: 配置文件格式错误");
}

#[test]
fn test_resource_limit_error() {
    let err = XoreError::ResourceLimit { resource: "内存".to_string(), current: 2048, max: 1024 };
    assert_eq!(err.to_string(), "超出资源限制: 内存 (当前: 2048, 最大: 1024)");
}

#[test]
fn test_ai_error() {
    let err = XoreError::AiError("模型加载失败".to_string());
    assert_eq!(err.to_string(), "AI模型错误: 模型加载失败");
}

#[test]
fn test_other_error() {
    let err = XoreError::Other("未知错误".to_string());
    assert_eq!(err.to_string(), "未知错误");
}

// ===== 新增错误类型测试 =====

#[test]
fn test_search_error() {
    let err = XoreError::SearchError("搜索索引损坏".to_string());
    assert_eq!(err.to_string(), "搜索错误: 搜索索引损坏");
    assert_eq!(err.error_code(), "SEARCH_ERROR");
}

#[test]
fn test_parse_error() {
    let err = XoreError::ParseError("CSV 格式无效".to_string());
    assert_eq!(err.to_string(), "解析错误: CSV 格式无效");
    assert_eq!(err.error_code(), "PARSE_ERROR");
}

#[test]
fn test_validation_error() {
    let err = XoreError::ValidationError("字段不能为空".to_string());
    assert_eq!(err.to_string(), "验证错误: 字段不能为空");
    assert_eq!(err.error_code(), "VALIDATION_ERROR");
}

#[test]
fn test_timeout_error() {
    let err = XoreError::Timeout("查询超时 30s".to_string());
    assert_eq!(err.to_string(), "超时: 查询超时 30s");
    assert_eq!(err.error_code(), "TIMEOUT");
}

#[test]
fn test_permission_denied_error() {
    let err = XoreError::PermissionDenied("/etc/passwd".to_string());
    assert_eq!(err.to_string(), "权限不足: /etc/passwd");
    assert_eq!(err.error_code(), "PERMISSION_DENIED");
}

// ===== 错误代码测试 =====

#[test]
fn test_all_error_codes() {
    assert_eq!(XoreError::IndexError("".to_string()).error_code(), "INDEX_ERROR");
    assert_eq!(XoreError::SearchError("".to_string()).error_code(), "SEARCH_ERROR");
    assert_eq!(XoreError::ProcessError("".to_string()).error_code(), "PROCESS_ERROR");
    assert_eq!(XoreError::SqlError("".to_string()).error_code(), "SQL_ERROR");
    assert_eq!(XoreError::ParseError("".to_string()).error_code(), "PARSE_ERROR");
    assert_eq!(XoreError::ValidationError("".to_string()).error_code(), "VALIDATION_ERROR");
    assert_eq!(XoreError::FileNotFound { path: "".to_string() }.error_code(), "FILE_NOT_FOUND");
    assert_eq!(XoreError::ConfigError("".to_string()).error_code(), "CONFIG_ERROR");
    assert_eq!(XoreError::HistoryError("".to_string()).error_code(), "HISTORY_ERROR");
    assert_eq!(XoreError::Timeout("".to_string()).error_code(), "TIMEOUT");
    assert_eq!(XoreError::PermissionDenied("".to_string()).error_code(), "PERMISSION_DENIED");
    assert_eq!(
        XoreError::ResourceLimit { resource: "".to_string(), current: 0, max: 0 }.error_code(),
        "RESOURCE_LIMIT"
    );
    assert_eq!(XoreError::AiError("".to_string()).error_code(), "AI_ERROR");
    assert_eq!(XoreError::Other("".to_string()).error_code(), "OTHER_ERROR");
}

// ===== 错误提示测试 =====

#[test]
fn test_hint_file_not_found() {
    let err = XoreError::FileNotFound { path: "/tmp/data.csv".to_string() };
    let hint = err.hint();
    assert!(hint.is_some());
    let hint = hint.unwrap();
    let formatted = hint.format();
    assert!(formatted.contains("/tmp/data.csv"));
    assert!(formatted.contains("ls"));
}

#[test]
fn test_hint_sql_error() {
    let err = XoreError::SqlError("near 'FORM': syntax error".to_string());
    let hint = err.hint();
    assert!(hint.is_some());
    let hint = hint.unwrap();
    assert!(hint.format().contains("xore agent explain"));
}

#[test]
fn test_hint_index_error() {
    let err = XoreError::IndexError("索引损坏".to_string());
    let hint = err.hint();
    assert!(hint.is_some());
    let hint = hint.unwrap();
    assert!(hint.format().contains("xore f --rebuild"));
}

#[test]
fn test_hint_config_error() {
    let err = XoreError::ConfigError("TOML 解析失败".to_string());
    let hint = err.hint();
    assert!(hint.is_some());
    let hint = hint.unwrap();
    assert!(hint.format().contains("configuration.md"));
}

#[test]
fn test_hint_none_for_other() {
    let err = XoreError::Other("未知错误".to_string());
    assert!(err.hint().is_none());
}

#[test]
fn test_hint_none_for_history_error() {
    let err = XoreError::HistoryError("历史记录损坏".to_string());
    // HistoryError 没有特定提示
    let _ = err.hint(); // 不应 panic
}

// ===== 错误格式化器测试 =====

#[test]
fn test_formatter_default() {
    let err = XoreError::FileNotFound { path: "test.csv".to_string() };
    let formatter = ErrorFormatter::default_format();
    let output = formatter.format(&err);

    assert!(output.contains("错误:"));
    assert!(output.contains("test.csv"));
    assert!(output.contains("提示:"));
}

#[test]
fn test_formatter_no_color() {
    let err = XoreError::SqlError("语法错误".to_string());
    let config = ErrorFormatterConfig { verbose: false, use_color: false, show_hints: true };
    let formatter = ErrorFormatter::new(config);
    let output = formatter.format(&err);

    // 不含 ANSI 转义序列
    assert!(!output.contains("\x1b["));
    assert!(output.contains("错误:"));
}

#[test]
fn test_formatter_verbose() {
    let err = XoreError::IndexError("索引损坏".to_string());
    let formatter = ErrorFormatter::verbose();
    let output = formatter.format(&err);

    assert!(output.contains("详细信息"));
    assert!(output.contains("INDEX_ERROR"));
    assert!(output.contains("xore f --rebuild"));
}

#[test]
fn test_formatter_no_hints() {
    let err = XoreError::FileNotFound { path: "test.csv".to_string() };
    let config = ErrorFormatterConfig { verbose: false, use_color: false, show_hints: false };
    let formatter = ErrorFormatter::new(config);
    let output = formatter.format(&err);

    assert!(!output.contains("提示:"));
}

#[test]
fn test_formatter_resource_limit() {
    let err = XoreError::ResourceLimit { resource: "内存".to_string(), current: 2048, max: 1024 };
    let config = ErrorFormatterConfig { verbose: false, use_color: false, show_hints: false };
    let formatter = ErrorFormatter::new(config);
    let output = formatter.format(&err);

    assert!(output.contains("内存"));
    assert!(output.contains("2048"));
    assert!(output.contains("1024"));
}

// ===== ErrorContext 测试 =====

#[test]
fn test_error_context_messages() {
    let ctx = ErrorContext::new().with_message("第一条上下文").with_message("第二条上下文");

    assert_eq!(ctx.messages().len(), 2);
    assert_eq!(ctx.messages()[0], "第一条上下文");
    assert_eq!(ctx.messages()[1], "第二条上下文");
}

#[test]
fn test_error_context_location() {
    let ctx = ErrorContext::new().with_location("src/main.rs", 42);

    let (file, line) = ctx.location().unwrap();
    assert_eq!(file, "src/main.rs");
    assert_eq!(line, 42);
}

#[test]
fn test_error_context_no_location() {
    let ctx = ErrorContext::new().with_message("测试");
    assert!(ctx.location().is_none());
}

// ===== ErrorHint 测试 =====

#[test]
fn test_error_hint_basic() {
    let hint = ErrorHint::new("这是一个提示");
    assert!(hint.format().contains("这是一个提示"));
}

#[test]
fn test_error_hint_with_command() {
    let hint = ErrorHint::new("提示").with_command("xore --help");
    let formatted = hint.format();
    assert!(formatted.contains("提示"));
    assert!(formatted.contains("xore --help"));
    assert!(formatted.contains("尝试运行"));
}

#[test]
fn test_error_hint_with_doc() {
    let hint = ErrorHint::new("提示").with_doc("docs/README.md");
    let formatted = hint.format();
    assert!(formatted.contains("docs/README.md"));
    assert!(formatted.contains("参考文档"));
}

#[test]
fn test_error_hint_full() {
    let hint =
        ErrorHint::new("完整提示").with_command("xore f --rebuild").with_doc("docs/guide.md");
    let formatted = hint.format();
    assert!(formatted.contains("完整提示"));
    assert!(formatted.contains("xore f --rebuild"));
    assert!(formatted.contains("docs/guide.md"));
}

// ===== ErrorChain 测试 =====

#[test]
fn test_error_chain_basic() {
    let err = XoreError::IndexError("索引构建失败".to_string());
    let chain = ErrorChain::new(err);
    assert!(chain.full_message().contains("索引构建失败"));
}

#[test]
fn test_error_chain_with_source() {
    let err = XoreError::IndexError("索引构建失败".to_string());
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "文件不存在");
    let chain = ErrorChain::new(err).with_source(io_err);

    let msg = chain.full_message();
    assert!(msg.contains("索引构建失败"));
    assert!(msg.contains("根本原因"));
    assert!(msg.contains("文件不存在"));
}

// ===== XoreErrorExt 测试 =====

#[test]
fn test_error_ext_context() {
    let err = XoreError::FileNotFound { path: "test.csv".to_string() };
    let err_with_ctx = XoreErrorExt::context(err, "加载配置时");
    assert!(err_with_ctx.to_string().contains("加载配置时"));
}

#[test]
fn test_error_ext_with_location() {
    let err = XoreError::FileNotFound { path: "test.csv".to_string() };
    let err_with_loc = XoreErrorExt::with_location(err, "src/main.rs", 100, "读取文件时");
    let msg = err_with_loc.to_string();
    assert!(msg.contains("src/main.rs"));
    assert!(msg.contains("100"));
    assert!(msg.contains("读取文件时"));
}

#[test]
fn test_from_string() {
    let err: XoreError = "测试错误".to_string().into();
    assert_eq!(err.to_string(), "测试错误");
}

#[test]
fn test_from_str() {
    let err: XoreError = "测试错误".into();
    assert_eq!(err.to_string(), "测试错误");
}

#[test]
fn test_result_ok() {
    fn returns_ok() -> Result<i32> {
        Ok(42)
    }
    let result = returns_ok();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn test_result_err() {
    let result: Result<i32> = Err(XoreError::Other("错误".to_string()));
    assert!(result.is_err());
}

#[test]
fn test_io_error_conversion() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "文件未找到");
    let xore_err: XoreError = io_err.into();
    assert!(xore_err.to_string().contains("文件未找到"));
}

#[test]
fn test_error_debug_format() {
    let err = XoreError::IndexError("测试".to_string());
    let debug_str = format!("{:?}", err);
    assert!(debug_str.contains("IndexError"));
}

#[test]
fn test_multiple_error_conversions() {
    // 测试从不同类型转换
    let err1: XoreError = "字符串错误".into();
    let err2: XoreError = String::from("String错误").into();

    assert!(err1.to_string().contains("字符串错误"));
    assert!(err2.to_string().contains("String错误"));
}
