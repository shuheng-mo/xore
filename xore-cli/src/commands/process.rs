//! Process 命令实现
//!
//! 提供数据处理功能，包括 SQL 查询、数据预览和质量检查。

use anyhow::{Context, Result};
use colored::*;
use std::path::Path;
use xore_process::{DataParser, DataProfiler};

use crate::ui::{Alignment, Column, Table, TableStyle, ICON_SUCCESS, ICON_TIP, ICON_WARNING};

/// 执行数据处理命令
pub fn execute(file: &str, query: Option<&str>, quality_check: bool) -> Result<()> {
    let path = Path::new(file);

    // 检查文件是否存在
    if !path.exists() {
        return Err(anyhow::anyhow!("文件不存在: {}", file));
    }

    let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();

    // 根据模式执行不同操作
    if quality_check {
        run_quality_check(path, &extension)?;
    } else if let Some(sql) = query {
        run_sql_query(path, sql, &extension)?;
    } else {
        run_data_preview(path, &extension)?;
    }

    Ok(())
}

/// 数据预览（使用 Polars）
fn run_data_preview(path: &Path, extension: &str) -> Result<()> {
    println!("{} {} {}\n", "📄".cyan(), "数据预览:".bold(), path.display().to_string().yellow());

    match extension {
        "csv" | "parquet" => preview_with_polars(path)?,
        "json" => preview_json(path)?,
        _ => {
            println!("{}", format!("不支持的文件格式: {}", extension).red());
            println!("{}", "支持的格式: csv, json, parquet".dimmed());
        }
    }

    Ok(())
}

/// 使用 Polars 预览数据
fn preview_with_polars(path: &Path) -> Result<()> {
    let parser = DataParser::new();

    // 读取数据
    let df = parser.read(path).with_context(|| format!("无法读取文件: {:?}", path))?;

    let total_rows = df.height();
    let total_cols = df.width();

    // 获取前 10 行
    let preview_df = df.head(Some(10));

    // 创建表格
    let column_names = preview_df.get_column_names();
    let columns: Vec<Column> = column_names.iter().map(|name| Column::new(name)).collect();

    let mut table = Table::new(columns).with_style(TableStyle::Simple);

    // 添加数据行
    for row_idx in 0..preview_df.height() {
        let row_data: Vec<String> = column_names
            .iter()
            .map(|col_name| {
                preview_df
                    .column(col_name)
                    .ok()
                    .and_then(|series| series.get(row_idx).ok())
                    .map(|val| format_anyvalue(&val))
                    .unwrap_or_else(|| "null".to_string())
            })
            .collect();

        table.add_row(row_data);
    }

    // 渲染表格
    print!("{}", table.render());

    // 显示统计信息
    println!(
        "\n显示前 {} 行 (共 {} 行, {} 列)",
        preview_df.height().to_string().cyan(),
        total_rows.to_string().cyan(),
        total_cols.to_string().cyan()
    );

    Ok(())
}

/// 格式化 Polars AnyValue 为字符串
fn format_anyvalue(val: &xore_process::AnyValue) -> String {
    use xore_process::AnyValue;

    match val {
        AnyValue::Null => "null".dimmed().to_string(),
        AnyValue::Boolean(b) => b.to_string(),
        AnyValue::String(s) => {
            if s.len() > 50 {
                format!("{}...", &s[..47])
            } else {
                s.to_string()
            }
        }
        AnyValue::UInt8(n) => n.to_string(),
        AnyValue::UInt16(n) => n.to_string(),
        AnyValue::UInt32(n) => n.to_string(),
        AnyValue::UInt64(n) => n.to_string(),
        AnyValue::Int8(n) => n.to_string(),
        AnyValue::Int16(n) => n.to_string(),
        AnyValue::Int32(n) => n.to_string(),
        AnyValue::Int64(n) => n.to_string(),
        AnyValue::Float32(n) => format!("{:.2}", n),
        AnyValue::Float64(n) => format!("{:.2}", n),
        _ => format!("{:?}", val),
    }
}

/// 预览 JSON 文件（保留原有实现）
fn preview_json(path: &Path) -> Result<()> {
    let content =
        std::fs::read_to_string(path).with_context(|| format!("无法读取文件: {:?}", path))?;

    // 尝试解析 JSON
    let value: serde_json::Value =
        serde_json::from_str(&content).with_context(|| "无效的 JSON 格式")?;

    match &value {
        serde_json::Value::Array(arr) => {
            if arr.is_empty() {
                println!("{}", "JSON 数组为空".yellow());
                return Ok(());
            }

            // 从第一个对象提取键作为表头
            if let Some(serde_json::Value::Object(first)) = arr.first() {
                let headers: Vec<&str> = first.keys().map(|s| s.as_str()).collect();
                let columns: Vec<Column> = headers.iter().map(|h| Column::new(h)).collect();

                let mut table = Table::new(columns).with_style(TableStyle::Simple);

                for obj in arr.iter().take(10) {
                    if let serde_json::Value::Object(map) = obj {
                        let cells: Vec<String> = headers
                            .iter()
                            .map(|h| map.get(*h).map(|v| format_json_value(v)).unwrap_or_default())
                            .collect();
                        table.add_row(cells);
                    }
                }

                print!("{}", table.render());
                println!(
                    "\n显示前 {} 行 (共 {} 行)",
                    arr.len().min(10).to_string().cyan(),
                    arr.len().to_string().cyan()
                );
            } else {
                // 非对象数组
                println!("JSON 数组内容:");
                for (i, item) in arr.iter().take(10).enumerate() {
                    println!("  {}: {}", i + 1, format_json_value(item));
                }
                if arr.len() > 10 {
                    println!("  ... 共 {} 项", arr.len());
                }
            }
        }
        serde_json::Value::Object(obj) => {
            println!("JSON 对象 ({} 个字段):\n", obj.len());
            for (key, value) in obj.iter().take(20) {
                println!("  {}: {}", key.cyan(), format_json_value(value).dimmed());
            }
            if obj.len() > 20 {
                println!("  ... 共 {} 个字段", obj.len());
            }
        }
        _ => {
            println!("JSON 值: {}", format_json_value(&value));
        }
    }

    Ok(())
}

/// 格式化 JSON 值为字符串
fn format_json_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => {
            if s.len() > 50 {
                format!("{}...", &s[..47])
            } else {
                s.clone()
            }
        }
        serde_json::Value::Array(arr) => format!("[{} items]", arr.len()),
        serde_json::Value::Object(obj) => format!("{{...}} ({} fields)", obj.len()),
    }
}

/// SQL 查询
fn run_sql_query(path: &Path, sql: &str, extension: &str) -> Result<()> {
    println!("{}", "⚙️  执行 SQL 查询...".cyan());
    println!("文件: {}", path.display().to_string().yellow());
    println!("查询: {}\n", sql.dimmed());

    match extension {
        "csv" | "parquet" => {
            // TODO: 实现实际的 SQL 查询 (Day 17-18)
            println!("{}", "SQL 查询功能即将推出，将使用 Polars SQL 引擎".yellow());
            println!("\n示例输出:");

            // 模拟输出
            let columns = vec![
                Column::new("column").with_alignment(Alignment::Left),
                Column::new("count").with_alignment(Alignment::Right),
            ];

            let table = Table::new(columns)
                .with_style(TableStyle::Simple)
                .row(["value1", "1234"])
                .row(["value2", "2345"])
                .row(["value3", "3456"]);

            print!("{}", table.render());
            println!("\n{} 处理完成 (模拟数据)", ICON_SUCCESS.green());
        }
        _ => {
            println!("{}", format!("SQL 查询不支持 {} 格式", extension).red());
        }
    }

    Ok(())
}

/// 数据质量检查（使用 Polars）
fn run_quality_check(path: &Path, extension: &str) -> Result<()> {
    println!(
        "{} {} {}\n",
        "🔍".cyan(),
        "数据质量检查:".bold(),
        path.display().to_string().yellow()
    );

    match extension {
        "csv" | "parquet" => quality_check_with_polars(path)?,
        "json" => quality_check_json(path)?,
        _ => {
            println!("{}", format!("质量检查不支持 {} 格式", extension).red());
            println!("{}", "支持的格式: csv, json, parquet".dimmed());
        }
    }

    Ok(())
}

/// 使用 Polars 进行质量检查
fn quality_check_with_polars(path: &Path) -> Result<()> {
    let parser = DataParser::new();
    let profiler = DataProfiler::new();

    // 读取数据
    let df = parser.read(path).with_context(|| format!("无法读取文件: {:?}", path))?;

    // 生成质量报告
    let report = profiler.profile(&df).with_context(|| "生成质量报告失败")?;

    // 基本信息
    println!("{}", "基本信息".bold());
    println!("  {} 总行数: {}", ICON_SUCCESS.green(), report.total_rows.to_string().cyan());
    println!("  {} 总列数: {}", ICON_SUCCESS.green(), report.total_columns.to_string().cyan());

    // 发现的问题
    println!("\n{}", "发现的问题".bold());

    let mut has_issues = false;

    // 缺失值
    if !report.missing_values.is_empty() {
        has_issues = true;
        println!("  {} 发现 {} 列存在缺失值", ICON_WARNING.yellow(), report.missing_values.len());
        for (name, stats) in report.missing_values.iter().take(5) {
            println!("    - {}: {:.1}% 缺失 ({} 行)", name.cyan(), stats.percentage, stats.count);
        }
    }

    // 重复行
    if report.duplicate_rows > 0 {
        has_issues = true;
        println!(
            "  {} 检测到 {} 行重复数据",
            ICON_WARNING.yellow(),
            report.duplicate_rows.to_string().yellow()
        );
    }

    if !has_issues {
        println!("  {} 未发现明显问题", ICON_SUCCESS.green());
    }

    // 建议
    println!("\n{}", "建议".bold());
    if report.duplicate_rows > 0 {
        println!("  {} 考虑去除重复行以减少数据冗余", ICON_TIP);
    }
    if !report.missing_values.is_empty() {
        println!("  {} 检查数据源，确保必填字段有值", ICON_TIP);
    }
    if !has_issues {
        println!("  {} 数据质量良好，可以继续处理", ICON_TIP);
    }

    Ok(())
}

/// JSON 文件质量检查（保留原有实现）
fn quality_check_json(path: &Path) -> Result<()> {
    let content =
        std::fs::read_to_string(path).with_context(|| format!("无法读取文件: {:?}", path))?;

    let value: serde_json::Value =
        serde_json::from_str(&content).with_context(|| "无效的 JSON 格式")?;

    // 基本信息
    println!("{}", "基本信息".bold());

    match &value {
        serde_json::Value::Array(arr) => {
            println!("  {} 类型: JSON 数组", ICON_SUCCESS.green());
            println!("  {} 元素数量: {}", ICON_SUCCESS.green(), arr.len().to_string().cyan());

            if let Some(serde_json::Value::Object(first)) = arr.first() {
                println!("  {} 字段数量: {}", ICON_SUCCESS.green(), first.len().to_string().cyan());
            }

            // 检查一致性
            println!("\n{}", "发现的问题".bold());
            let mut inconsistent = 0;
            let first_keys: Option<std::collections::HashSet<&String>> =
                arr.first().and_then(|v| v.as_object()).map(|obj| obj.keys().collect());

            if let Some(ref keys) = first_keys {
                for item in arr.iter().skip(1) {
                    if let serde_json::Value::Object(obj) = item {
                        let item_keys: std::collections::HashSet<&String> = obj.keys().collect();
                        if item_keys != *keys {
                            inconsistent += 1;
                        }
                    }
                }
            }

            if inconsistent > 0 {
                println!(
                    "  {} {} 个元素字段不一致",
                    ICON_WARNING.yellow(),
                    inconsistent.to_string().yellow()
                );
            } else {
                println!("  {} 所有元素结构一致", ICON_SUCCESS.green());
            }
        }
        serde_json::Value::Object(obj) => {
            println!("  {} 类型: JSON 对象", ICON_SUCCESS.green());
            println!("  {} 字段数量: {}", ICON_SUCCESS.green(), obj.len().to_string().cyan());
            println!("\n{}", "发现的问题".bold());
            println!("  {} 未发现问题", ICON_SUCCESS.green());
        }
        _ => {
            println!("  {} 类型: JSON 原始值", ICON_SUCCESS.green());
            println!("\n{}", "发现的问题".bold());
            println!("  {} 数据为原始值，非结构化数据", ICON_WARNING.yellow());
        }
    }

    // 建议
    println!("\n{}", "建议".bold());
    println!("  {} JSON 数据通常可直接处理", ICON_TIP);

    Ok(())
}
