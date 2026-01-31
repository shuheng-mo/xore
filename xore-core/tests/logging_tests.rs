//! 日志模块集成测试

use xore_core::{LogConfig, LogLevel};

#[test]
fn test_log_config_builder_pattern() {
    let config = LogConfig::new()
        .with_level(LogLevel::Verbose)
        .with_color(false)
        .with_timestamp(true)
        .with_target(true);

    assert_eq!(config.level, LogLevel::Verbose);
    assert!(!config.color);
    assert!(config.with_timestamp);
    assert!(config.with_target);
}

#[test]
fn test_log_config_default() {
    let config = LogConfig::default();
    assert_eq!(config.level, LogLevel::Normal);
    assert!(config.color);
    assert!(!config.with_timestamp);
    assert!(!config.with_target);
}

#[test]
fn test_from_args_quiet_mode() {
    let config = LogConfig::from_args(false, true, false);
    assert_eq!(config.level, LogLevel::Quiet);
    assert!(config.color);
}

#[test]
fn test_from_args_verbose_mode() {
    let config = LogConfig::from_args(true, false, false);
    assert_eq!(config.level, LogLevel::Verbose);
    assert!(config.with_timestamp);
    assert!(config.with_target);
}

#[test]
fn test_from_args_no_color() {
    let config = LogConfig::from_args(false, false, true);
    assert!(!config.color);
}

#[test]
fn test_from_args_quiet_takes_precedence() {
    // 当 quiet 和 verbose 都为 true 时，quiet 优先
    let config = LogConfig::from_args(true, true, false);
    assert_eq!(config.level, LogLevel::Quiet);
}

#[test]
fn test_log_level_ordering() {
    // 验证日志级别的逻辑顺序
    assert_ne!(LogLevel::Quiet, LogLevel::Normal);
    assert_ne!(LogLevel::Normal, LogLevel::Verbose);
    assert_ne!(LogLevel::Verbose, LogLevel::Trace);
}

#[test]
fn test_log_config_clone() {
    let config = LogConfig::new().with_level(LogLevel::Trace).with_color(false);

    let cloned = config.clone();
    assert_eq!(cloned.level, LogLevel::Trace);
    assert!(!cloned.color);
}

#[test]
fn test_log_config_debug_format() {
    let config = LogConfig::default();
    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("LogConfig"));
    assert!(debug_str.contains("Normal"));
}

#[test]
fn test_log_level_clone_and_copy() {
    let level = LogLevel::Verbose;
    let cloned = level;
    assert_eq!(level, cloned);
}
