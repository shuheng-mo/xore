//! 终端检测工具
//!
//! 提供终端环境检测功能，包括 TTY 检测、颜色支持和终端宽度。

#![allow(dead_code)]

use std::io::IsTerminal;

/// 终端检测工具
pub struct Terminal;

impl Terminal {
    /// 检测标准输出是否为终端（TTY）
    pub fn is_tty() -> bool {
        std::io::stdout().is_terminal()
    }

    /// 检测标准错误是否为终端（TTY）
    pub fn is_stderr_tty() -> bool {
        std::io::stderr().is_terminal()
    }

    /// 获取终端宽度
    ///
    /// 如果无法检测到终端宽度，返回默认值 80
    pub fn width() -> usize {
        Self::size().map(|(w, _)| w).unwrap_or(80)
    }

    /// 获取终端高度
    ///
    /// 如果无法检测到终端高度，返回默认值 24
    pub fn height() -> usize {
        Self::size().map(|(_, h)| h).unwrap_or(24)
    }

    /// 获取终端尺寸 (宽度, 高度)
    ///
    /// 返回 None 如果无法检测
    pub fn size() -> Option<(usize, usize)> {
        #[cfg(unix)]
        {
            Self::unix_terminal_size()
        }
        #[cfg(windows)]
        {
            Self::windows_terminal_size()
        }
        #[cfg(not(any(unix, windows)))]
        {
            None
        }
    }

    /// 检测终端是否支持颜色输出
    pub fn supports_color() -> bool {
        // 如果不是终端，不支持颜色
        if !Self::is_tty() {
            return false;
        }

        // 检查 NO_COLOR 环境变量（https://no-color.org/）
        if std::env::var("NO_COLOR").is_ok() {
            return false;
        }

        // 检查 TERM 环境变量
        if let Ok(term) = std::env::var("TERM") {
            if term == "dumb" {
                return false;
            }
        }

        // 检查 CLICOLOR_FORCE 环境变量
        if std::env::var("CLICOLOR_FORCE").map(|v| v != "0").unwrap_or(false) {
            return true;
        }

        // 默认情况下，如果是终端则支持颜色
        true
    }

    #[cfg(unix)]
    fn unix_terminal_size() -> Option<(usize, usize)> {
        use std::os::unix::io::AsRawFd;

        let fd = std::io::stdout().as_raw_fd();
        let mut winsize: libc::winsize = unsafe { std::mem::zeroed() };

        let result = unsafe { libc::ioctl(fd, libc::TIOCGWINSZ, &mut winsize) };

        if result == 0 && winsize.ws_col > 0 && winsize.ws_row > 0 {
            Some((winsize.ws_col as usize, winsize.ws_row as usize))
        } else {
            None
        }
    }

    #[cfg(windows)]
    fn windows_terminal_size() -> Option<(usize, usize)> {
        // Windows 简化实现
        // 在实际项目中可以使用 windows-sys crate 获取真实尺寸
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_width() {
        let width = Terminal::width();
        assert!(width > 0);
    }

    #[test]
    fn test_terminal_height() {
        let height = Terminal::height();
        assert!(height > 0);
    }

    #[test]
    fn test_supports_color() {
        // 这个测试的结果取决于运行环境
        let _ = Terminal::supports_color();
    }
}
