//! 错误处理模块单元测试

use xore_core::{Result, XoreError};

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
