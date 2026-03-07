//! Process 命令实现
//!
//! 提供数据处理功能，包括 SQL 查询、数据预览和质量检查。

use anyhow::{Context, Result};
use colored::*;
use std::path::Path;
use xore_process::{DataExporter, DataParser, DataProfiler, ExportFormat, SqlEngine};

use crate::ui::{Column, Table, TableStyle, ICON_SUCCESS, ICON_TIP, ICON_WARNING};

/// 执行数据处理命令
pub fn execute(
    file: &str,
    query: Option<&str>,
    quality_check: bool,
    output: Option<&str>,
    format: Option<&str>,
) -> Result<()> {
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
        run_sql_query(path, sql, &extension, output, format)?;
    } else if let Some(output_path) = output {
        // 无 SQL 查询但指定了输出路径，执行格式转换
        run_format_convert(path, &extension, output_path, format)?;
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

/// 格式转换（无 SQL，直接读取文件并导出为目标格式）
fn run_format_convert(
    path: &Path,
    extension: &str,
    output_path: &str,
    format: Option<&str>,
) -> Result<()> {
    println!("{} {} {}\n", "🔄".cyan(), "格式转换:".bold(), path.display().to_string().yellow());

    match extension {
        "csv" | "parquet" => {
            let parser = DataParser::new();
            let mut df = parser.read(path).with_context(|| format!("无法读取文件: {:?}", path))?;
            export_dataframe(&mut df, output_path, format)?;
        }
        _ => {
            println!("{}", format!("格式转换不支持 {} 格式", extension).red());
            println!("{}", "支持的输入格式: csv, parquet".dimmed());
        }
    }

    Ok(())
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
                            .map(|h| map.get(*h).map(format_json_value).unwrap_or_default())
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
fn run_sql_query(
    path: &Path,
    sql: &str,
    extension: &str,
    output: Option<&str>,
    format: Option<&str>,
) -> Result<()> {
    println!("{} {} SQL 查询...\n", "⚙️".cyan(), "执行".bold());
    println!("文件: {}", path.display().to_string().yellow());
    println!("查询: {}\n", sql.dimmed());

    match extension {
        "csv" | "parquet" => {
            // 创建 SQL 引擎
            let mut engine = SqlEngine::new();

            // 注册表（使用文件名作为表名，去除扩展名）
            let table_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("this");

            engine
                .register_table(table_name, path)
                .with_context(|| format!("无法注册表 '{}'", table_name))?;

            // 执行查询
            let mut result = engine.execute(sql).with_context(|| "SQL 查询执行失败")?;

            // 如果指定了输出文件，导出结果
            if let Some(output_path) = output {
                export_dataframe(&mut result, output_path, format)?;
            } else {
                // 否则渲染到终端
                render_dataframe_as_table(&result)?;

                println!(
                    "\n{} 查询完成 ({} 行, {} 列)",
                    ICON_SUCCESS.green(),
                    result.height().to_string().cyan(),
                    result.width().to_string().cyan()
                );
            }
        }
        _ => {
            println!("{}", format!("SQL 查询不支持 {} 格式", extension).red());
            println!("{}", "支持的格式: csv, parquet".dimmed());
        }
    }

    Ok(())
}

/// 导出 DataFrame 到文件
fn export_dataframe(
    df: &mut xore_process::DataFrame,
    output_path: &str,
    format: Option<&str>,
) -> Result<()> {
    let exporter = DataExporter::new();
    let path = Path::new(output_path);

    // 解析格式
    let export_format = if let Some(fmt) = format {
        match fmt.to_lowercase().as_str() {
            "csv" => Some(ExportFormat::Csv),
            "json" => Some(ExportFormat::Json),
            "parquet" => Some(ExportFormat::Parquet),
            "arrow" => Some(ExportFormat::Arrow),
            _ => return Err(anyhow::anyhow!("不支持的导出格式: {}", fmt)),
        }
    } else {
        None // 从文件扩展名推断
    };

    println!("{} 导出数据到 {}...", "💾".cyan(), output_path.yellow());

    let bytes = exporter
        .export(df, path, export_format)
        .with_context(|| format!("导出失败: {}", output_path))?;

    println!(
        "{} 导出完成 ({} 行, {} 列, {} 字节)",
        ICON_SUCCESS.green(),
        df.height().to_string().cyan(),
        df.width().to_string().cyan(),
        bytes.to_string().cyan()
    );

    Ok(())
}

/// 将 DataFrame 渲染为表格
fn render_dataframe_as_table(df: &xore_process::DataFrame) -> Result<()> {
    let column_names = df.get_column_names();
    let columns: Vec<Column> = column_names.iter().map(|name| Column::new(name)).collect();

    let mut table = Table::new(columns).with_style(TableStyle::Simple);

    // 添加数据行（最多显示 100 行）
    let max_rows = df.height().min(100);
    for row_idx in 0..max_rows {
        let row_data: Vec<String> = column_names
            .iter()
            .map(|col_name| {
                df.column(col_name)
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

    // 如果行数超过 100，显示提示
    if df.height() > 100 {
        println!("\n{} 仅显示前 100 行，共 {} 行", ICON_TIP.blue(), df.height().to_string().cyan());
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
    use xore_process::Severity;

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
            let color_fn = if stats.percentage > 50.0 {
                |s: String| s.red()
            } else if stats.percentage > 10.0 {
                |s: String| s.yellow()
            } else {
                |s: String| s.normal()
            };
            println!(
                "    - {}: {} 缺失 ({} 行)",
                name.cyan(),
                color_fn(format!("{:.1}%", stats.percentage)),
                stats.count
            );
        }
    }

    // 重复行
    if report.duplicate_rows > 0 {
        has_issues = true;
        let dup_percentage = if report.total_rows > 0 {
            (report.duplicate_rows as f64 / report.total_rows as f64) * 100.0
        } else {
            0.0
        };
        println!(
            "  {} 检测到 {} 行重复数据 ({:.1}%)",
            ICON_WARNING.yellow(),
            report.duplicate_rows.to_string().yellow(),
            dup_percentage
        );
    }

    // 离群值
    if !report.outliers.is_empty() {
        has_issues = true;
        println!("  {} 发现 {} 列存在离群值", ICON_WARNING.yellow(), report.outliers.len());
        for (name, info) in report.outliers.iter().take(5) {
            println!("    - {}: {} 个离群值 ({:.1}%)", name.cyan(), info.count, info.percentage);
        }
    }

    if !has_issues {
        println!("  {} 未发现明显问题", ICON_SUCCESS.green());
    }

    // 智能建议
    if !report.suggestions.is_empty() {
        println!("\n{}", "智能建议".bold());

        // 按严重程度分组显示
        let errors: Vec<_> =
            report.suggestions.iter().filter(|s| s.severity == Severity::Error).collect();
        let warnings: Vec<_> =
            report.suggestions.iter().filter(|s| s.severity == Severity::Warning).collect();
        let infos: Vec<_> =
            report.suggestions.iter().filter(|s| s.severity == Severity::Info).collect();

        // 显示错误级别建议
        for suggestion in errors {
            println!("  {} {}", "❌".red(), suggestion.message.red());
        }

        // 显示警告级别建议
        for suggestion in warnings {
            println!("  {} {}", ICON_WARNING.yellow(), suggestion.message.yellow());
        }

        // 显示信息级别建议（最多显示 3 条）
        for suggestion in infos.iter().take(3) {
            println!("  {} {}", ICON_TIP.blue(), suggestion.message);
        }
    } else {
        println!("\n{}", "智能建议".bold());
        println!("  {} 数据质量良好，可以继续处理", ICON_TIP.blue());
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
