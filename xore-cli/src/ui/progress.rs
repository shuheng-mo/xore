//! 进度显示组件
//!
//! 提供 Spinner（不确定进度）和 ProgressBar（确定进度）组件。

use indicatif::{ProgressBar as IndicatifBar, ProgressStyle};
use std::time::Duration;

/// 旋转器 - 用于不确定进度的任务
pub struct Spinner {
    inner: IndicatifBar,
}

impl Spinner {
    /// 创建新的 Spinner
    pub fn new(message: &str) -> Self {
        let pb = IndicatifBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.cyan} {msg}")
                .unwrap()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(Duration::from_millis(80));

        Self { inner: pb }
    }

    /// 创建带自定义颜色的 Spinner
    pub fn with_style(message: &str, color: &str) -> Self {
        let pb = IndicatifBar::new_spinner();
        let template = format!("{{spinner:.{}}} {{msg}}", color);
        pb.set_style(
            ProgressStyle::default_spinner().template(&template).unwrap().tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(Duration::from_millis(80));

        Self { inner: pb }
    }

    /// 更新消息
    pub fn set_message(&self, message: &str) {
        self.inner.set_message(message.to_string());
    }

    /// 完成并清除
    pub fn finish(&self) {
        self.inner.finish_and_clear();
    }

    /// 完成并显示最终消息
    pub fn finish_with_message(&self, message: &str) {
        self.inner.finish_with_message(message.to_string());
    }
}

/// 进度条 - 用于确定进度的任务
pub struct ProgressBar {
    inner: IndicatifBar,
}

impl ProgressBar {
    /// 创建新的进度条
    pub fn new(total: u64) -> Self {
        let pb = IndicatifBar::new(total);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) {msg}")
                .unwrap()
                .progress_chars("━━╺"),
        );

        Self { inner: pb }
    }

    /// 创建用于字节进度的进度条（显示速度 MB/s）
    pub fn new_bytes(total: u64) -> Self {
        let pb = IndicatifBar::new(total);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}) {msg}")
                .unwrap()
                .progress_chars("━━╺"),
        );

        Self { inner: pb }
    }

    /// 创建用于文件数量进度的进度条
    pub fn new_files(total: u64) -> Self {
        let pb = IndicatifBar::new(total);
        pb.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} files ({percent}%) {msg}",
                )
                .unwrap()
                .progress_chars("━━╺"),
        );

        Self { inner: pb }
    }

    /// 设置进度
    pub fn set_position(&self, pos: u64) {
        self.inner.set_position(pos);
    }

    /// 增加进度
    pub fn inc(&self, delta: u64) {
        self.inner.inc(delta);
    }

    /// 更新消息
    pub fn set_message(&self, message: &str) {
        self.inner.set_message(message.to_string());
    }

    /// 完成进度条
    pub fn finish(&self) {
        self.inner.finish();
    }

    /// 完成并清除
    pub fn finish_and_clear(&self) {
        self.inner.finish_and_clear();
    }

    /// 完成并显示最终消息
    pub fn finish_with_message(&self, message: &str) {
        self.inner.finish_with_message(message.to_string());
    }

    /// 获取当前位置
    pub fn position(&self) -> u64 {
        self.inner.position()
    }

    /// 获取总量
    pub fn length(&self) -> Option<u64> {
        self.inner.length()
    }
}

/// 多进度条管理器
pub struct MultiProgress {
    inner: indicatif::MultiProgress,
}

impl MultiProgress {
    /// 创建新的多进度条管理器
    pub fn new() -> Self {
        Self { inner: indicatif::MultiProgress::new() }
    }

    /// 添加一个 Spinner
    pub fn add_spinner(&self, message: &str) -> Spinner {
        let pb = self.inner.add(IndicatifBar::new_spinner());
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.cyan} {msg}")
                .unwrap()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(Duration::from_millis(80));

        Spinner { inner: pb }
    }

    /// 添加一个进度条
    pub fn add_progress(&self, total: u64) -> ProgressBar {
        let pb = self.inner.add(IndicatifBar::new(total));
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) {msg}")
                .unwrap()
                .progress_chars("━━╺"),
        );

        ProgressBar { inner: pb }
    }
}

impl Default for MultiProgress {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spinner_creation() {
        let spinner = Spinner::new("Testing...");
        spinner.set_message("Updated");
        spinner.finish();
    }

    #[test]
    fn test_progress_bar_creation() {
        let pb = ProgressBar::new(100);
        pb.set_position(50);
        pb.inc(10);
        assert_eq!(pb.position(), 60);
        pb.finish();
    }

    #[test]
    fn test_progress_bar_bytes() {
        let pb = ProgressBar::new_bytes(1024 * 1024);
        pb.set_position(512 * 1024);
        pb.finish();
    }
}
