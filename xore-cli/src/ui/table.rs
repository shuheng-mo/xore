//! 表格组件
//!
//! 提供 CLI 表格渲染功能，支持多种样式和对齐方式。

use std::fmt;

/// 列对齐方式
#[derive(Debug, Clone, Copy, Default)]
pub enum Alignment {
    #[default]
    Left,
    Right,
    Center,
}

/// 表格样式
#[derive(Debug, Clone, Copy, Default)]
pub enum TableStyle {
    /// 简单样式（无边框，仅用空格分隔）
    #[default]
    Simple,
    /// 带边框样式
    Bordered,
    /// Markdown 样式
    Markdown,
    /// 紧凑样式（无表头分隔线）
    Compact,
}

/// 列定义
#[derive(Debug, Clone)]
pub struct Column {
    /// 列标题
    pub header: String,
    /// 对齐方式
    pub alignment: Alignment,
    /// 最小宽度
    pub min_width: Option<usize>,
    /// 最大宽度
    pub max_width: Option<usize>,
}

impl Column {
    /// 创建新列
    pub fn new(header: &str) -> Self {
        Self {
            header: header.to_string(),
            alignment: Alignment::Left,
            min_width: None,
            max_width: None,
        }
    }

    /// 设置对齐方式
    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// 设置最小宽度
    pub fn with_min_width(mut self, width: usize) -> Self {
        self.min_width = Some(width);
        self
    }

    /// 设置最大宽度
    pub fn with_max_width(mut self, width: usize) -> Self {
        self.max_width = Some(width);
        self
    }
}

/// 表格
pub struct Table {
    columns: Vec<Column>,
    rows: Vec<Vec<String>>,
    style: TableStyle,
}

impl Table {
    /// 创建新表格
    pub fn new(columns: Vec<Column>) -> Self {
        Self { columns, rows: Vec::new(), style: TableStyle::default() }
    }

    /// 从标题字符串创建表格
    pub fn from_headers(headers: &[&str]) -> Self {
        let columns = headers.iter().map(|h| Column::new(h)).collect();
        Self::new(columns)
    }

    /// 设置表格样式
    pub fn with_style(mut self, style: TableStyle) -> Self {
        self.style = style;
        self
    }

    /// 添加一行数据
    pub fn add_row<I, S>(&mut self, row: I)
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let row_vec: Vec<String> = row.into_iter().map(|s| s.as_ref().to_string()).collect();
        self.rows.push(row_vec);
    }

    /// 添加一行数据（链式调用）
    pub fn row<I, S>(mut self, row: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.add_row(row);
        self
    }

    /// 计算每列的实际宽度
    fn calculate_widths(&self) -> Vec<usize> {
        let mut widths: Vec<usize> = self.columns.iter().map(|c| c.header.len()).collect();

        for row in &self.rows {
            for (i, cell) in row.iter().enumerate() {
                if i < widths.len() {
                    widths[i] = widths[i].max(cell.len());
                }
            }
        }

        // 应用最小/最大宽度约束
        for (i, col) in self.columns.iter().enumerate() {
            if let Some(min) = col.min_width {
                widths[i] = widths[i].max(min);
            }
            if let Some(max) = col.max_width {
                widths[i] = widths[i].min(max);
            }
        }

        widths
    }

    /// 格式化单元格内容
    fn format_cell(&self, content: &str, width: usize, alignment: Alignment) -> String {
        let content = if content.len() > width {
            format!("{}...", &content[..width.saturating_sub(3)])
        } else {
            content.to_string()
        };

        match alignment {
            Alignment::Left => format!("{:<width$}", content, width = width),
            Alignment::Right => format!("{:>width$}", content, width = width),
            Alignment::Center => format!("{:^width$}", content, width = width),
        }
    }

    /// 渲染表格为字符串
    pub fn render(&self) -> String {
        let widths = self.calculate_widths();
        let mut output = String::new();

        match self.style {
            TableStyle::Simple => self.render_simple(&widths, &mut output),
            TableStyle::Bordered => self.render_bordered(&widths, &mut output),
            TableStyle::Markdown => self.render_markdown(&widths, &mut output),
            TableStyle::Compact => self.render_compact(&widths, &mut output),
        }

        output
    }

    fn render_simple(&self, widths: &[usize], output: &mut String) {
        // 表头
        let header: Vec<String> = self
            .columns
            .iter()
            .enumerate()
            .map(|(i, col)| self.format_cell(&col.header, widths[i], col.alignment))
            .collect();
        output.push_str(&header.join("  "));
        output.push('\n');

        // 分隔线
        let separator: Vec<String> = widths.iter().map(|&w| "-".repeat(w)).collect();
        output.push_str(&separator.join("  "));
        output.push('\n');

        // 数据行
        for row in &self.rows {
            let formatted: Vec<String> = row
                .iter()
                .enumerate()
                .map(|(i, cell)| {
                    let alignment = self.columns.get(i).map(|c| c.alignment).unwrap_or_default();
                    let width = *widths.get(i).unwrap_or(&0);
                    self.format_cell(cell, width, alignment)
                })
                .collect();
            output.push_str(&formatted.join("  "));
            output.push('\n');
        }
    }

    fn render_bordered(&self, widths: &[usize], output: &mut String) {
        let total_width: usize = widths.iter().sum::<usize>() + widths.len() * 3 + 1;

        // 顶部边框
        output.push_str(&"─".repeat(total_width));
        output.push('\n');

        // 表头
        output.push('│');
        for (i, col) in self.columns.iter().enumerate() {
            output.push(' ');
            output.push_str(&self.format_cell(&col.header, widths[i], col.alignment));
            output.push_str(" │");
        }
        output.push('\n');

        // 表头分隔线
        output.push_str(&"─".repeat(total_width));
        output.push('\n');

        // 数据行
        for row in &self.rows {
            output.push('│');
            for (i, cell) in row.iter().enumerate() {
                let alignment = self.columns.get(i).map(|c| c.alignment).unwrap_or_default();
                let width = *widths.get(i).unwrap_or(&0);
                output.push(' ');
                output.push_str(&self.format_cell(cell, width, alignment));
                output.push_str(" │");
            }
            output.push('\n');
        }

        // 底部边框
        output.push_str(&"─".repeat(total_width));
        output.push('\n');
    }

    fn render_markdown(&self, widths: &[usize], output: &mut String) {
        // 表头
        output.push('|');
        for (i, col) in self.columns.iter().enumerate() {
            output.push(' ');
            output.push_str(&self.format_cell(&col.header, widths[i], col.alignment));
            output.push_str(" |");
        }
        output.push('\n');

        // 分隔线（包含对齐标记）
        output.push('|');
        for (i, col) in self.columns.iter().enumerate() {
            let width = widths[i];
            let sep = match col.alignment {
                Alignment::Left => format!(":{}-", "-".repeat(width)),
                Alignment::Right => format!("-{}:", "-".repeat(width)),
                Alignment::Center => format!(":{}:", "-".repeat(width)),
            };
            output.push_str(&sep);
            output.push('|');
        }
        output.push('\n');

        // 数据行
        for row in &self.rows {
            output.push('|');
            for (i, cell) in row.iter().enumerate() {
                let alignment = self.columns.get(i).map(|c| c.alignment).unwrap_or_default();
                let width = *widths.get(i).unwrap_or(&0);
                output.push(' ');
                output.push_str(&self.format_cell(cell, width, alignment));
                output.push_str(" |");
            }
            output.push('\n');
        }
    }

    fn render_compact(&self, widths: &[usize], output: &mut String) {
        // 表头
        let header: Vec<String> = self
            .columns
            .iter()
            .enumerate()
            .map(|(i, col)| self.format_cell(&col.header, widths[i], col.alignment))
            .collect();
        output.push_str(&header.join("  "));
        output.push('\n');

        // 数据行（无分隔线）
        for row in &self.rows {
            let formatted: Vec<String> = row
                .iter()
                .enumerate()
                .map(|(i, cell)| {
                    let alignment = self.columns.get(i).map(|c| c.alignment).unwrap_or_default();
                    let width = *widths.get(i).unwrap_or(&0);
                    self.format_cell(cell, width, alignment)
                })
                .collect();
            output.push_str(&formatted.join("  "));
            output.push('\n');
        }
    }

    /// 获取行数
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// 获取列数
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }
}

impl fmt::Display for Table {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.render())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_table() {
        let mut table = Table::from_headers(&["Name", "Age", "City"]);
        table.add_row(["Alice", "25", "Beijing"]);
        table.add_row(["Bob", "30", "Shanghai"]);

        let output = table.render();
        assert!(output.contains("Name"));
        assert!(output.contains("Alice"));
        assert!(output.contains("Bob"));
    }

    #[test]
    fn test_markdown_table() {
        let table = Table::from_headers(&["Col1", "Col2"])
            .with_style(TableStyle::Markdown)
            .row(["A", "B"])
            .row(["C", "D"]);

        let output = table.render();
        assert!(output.contains("|"));
        assert!(output.contains("---"));
    }

    #[test]
    fn test_alignment() {
        let columns = vec![
            Column::new("Left").with_alignment(Alignment::Left),
            Column::new("Right").with_alignment(Alignment::Right),
            Column::new("Center").with_alignment(Alignment::Center),
        ];
        let mut table = Table::new(columns);
        table.add_row(["A", "B", "C"]);

        let _ = table.render();
    }

    #[test]
    fn test_chain_methods() {
        let table = Table::from_headers(&["X", "Y"])
            .with_style(TableStyle::Simple)
            .row(["1", "2"])
            .row(["3", "4"]);

        assert_eq!(table.row_count(), 2);
        assert_eq!(table.column_count(), 2);
    }
}
