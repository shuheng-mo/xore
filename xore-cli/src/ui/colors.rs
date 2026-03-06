//! 颜色方案和样式定义
//!
//! 提供统一的 CLI 输出颜色风格和图标常量。

#![allow(dead_code)]

use colored::{ColoredString, Colorize};

/// 成功图标
pub const ICON_SUCCESS: &str = "✓";
/// 警告图标
pub const ICON_WARNING: &str = "⚠";
/// 错误图标
pub const ICON_ERROR: &str = "✗";
/// 信息图标
pub const ICON_INFO: &str = "ℹ";
/// 等待图标
pub const ICON_PENDING: &str = "⏳";
/// 提示图标
pub const ICON_TIP: &str = "💡";
/// 文件图标
pub const ICON_FILE: &str = "📄";
/// 文件夹图标
pub const ICON_FOLDER: &str = "📁";

/// 颜色方案，提供统一的着色方法
pub struct ColorScheme;

impl ColorScheme {
    /// 成功样式（绿色）
    pub fn success<S: AsRef<str>>(text: S) -> ColoredString {
        text.as_ref().green()
    }

    /// 警告样式（黄色）
    pub fn warning<S: AsRef<str>>(text: S) -> ColoredString {
        text.as_ref().yellow()
    }

    /// 错误样式（红色）
    pub fn error<S: AsRef<str>>(text: S) -> ColoredString {
        text.as_ref().red()
    }

    /// 信息样式（蓝色）
    pub fn info<S: AsRef<str>>(text: S) -> ColoredString {
        text.as_ref().blue()
    }

    /// 高亮样式（品红色/洋红色）
    pub fn highlight<S: AsRef<str>>(text: S) -> ColoredString {
        text.as_ref().magenta()
    }

    /// 暗淡样式（灰色）
    pub fn dimmed<S: AsRef<str>>(text: S) -> ColoredString {
        text.as_ref().dimmed()
    }

    /// 数字样式（青色）
    pub fn number<S: AsRef<str>>(text: S) -> ColoredString {
        text.as_ref().cyan()
    }

    /// 加粗样式
    pub fn bold<S: AsRef<str>>(text: S) -> ColoredString {
        text.as_ref().bold()
    }

    /// 成功消息（带图标）
    pub fn success_msg<S: AsRef<str>>(text: S) -> String {
        format!("{} {}", ICON_SUCCESS.green(), text.as_ref().green())
    }

    /// 警告消息（带图标）
    pub fn warning_msg<S: AsRef<str>>(text: S) -> String {
        format!("{} {}", ICON_WARNING.yellow(), text.as_ref().yellow())
    }

    /// 错误消息（带图标）
    pub fn error_msg<S: AsRef<str>>(text: S) -> String {
        format!("{} {}", ICON_ERROR.red(), text.as_ref().red())
    }

    /// 信息消息（带图标）
    pub fn info_msg<S: AsRef<str>>(text: S) -> String {
        format!("{} {}", ICON_INFO.blue(), text.as_ref().blue())
    }

    /// 提示消息（带图标）
    pub fn tip_msg<S: AsRef<str>>(text: S) -> String {
        format!("{} {}", ICON_TIP, text.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_scheme() {
        // 测试各种颜色方法不会 panic
        let _ = ColorScheme::success("test");
        let _ = ColorScheme::warning("test");
        let _ = ColorScheme::error("test");
        let _ = ColorScheme::info("test");
        let _ = ColorScheme::highlight("test");
        let _ = ColorScheme::dimmed("test");
        let _ = ColorScheme::number("test");
        let _ = ColorScheme::bold("test");
    }

    #[test]
    fn test_message_formats() {
        let success = ColorScheme::success_msg("Done");
        assert!(success.contains(ICON_SUCCESS));

        let warning = ColorScheme::warning_msg("Caution");
        assert!(warning.contains(ICON_WARNING));

        let error = ColorScheme::error_msg("Failed");
        assert!(error.contains(ICON_ERROR));
    }
}
