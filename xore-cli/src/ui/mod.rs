//! UI 组件模块
//!
//! 提供 CLI 输出的各种组件：
//! - `colors`: 颜色方案和样式
//! - `progress`: 进度条和 Spinner
//! - `table`: 表格渲染
//! - `terminal`: 终端检测工具

pub mod colors;
pub mod progress;
pub mod table;
pub mod terminal;

pub use colors::{
    ColorScheme, ICON_ERROR, ICON_INFO, ICON_PENDING, ICON_SUCCESS, ICON_TIP, ICON_WARNING,
};
pub use progress::{MultiProgress, ProgressBar, Spinner};
pub use table::{Alignment, Column, Table, TableStyle};
pub use terminal::Terminal;
