//! Benchmark 命令实现
//!
//! 提供性能基准测试功能，用于测量各组件的性能。

use anyhow::Result;
use clap::ValueEnum;
use colored::*;
use std::io::{Read, Write};
use std::time::{Duration, Instant};

use crate::ui::{ColorScheme, Table, ICON_PENDING, ICON_SUCCESS};
use xore_search::{FileScanner, ScanConfig};

/// 基准测试套件类型
#[derive(Debug, Clone, Copy, ValueEnum, Default)]
pub enum BenchmarkSuite {
    /// 运行所有基准测试
    #[default]
    All,
    /// 文件扫描性能测试
    Scan,
    /// 搜索性能测试（待实现）
    Search,
    /// 数据处理性能测试（待实现）
    Process,
    /// I/O 吞吐量测试
    Io,
    /// 内存分配性能测试
    Alloc,
}

/// 输出格式
#[derive(Debug, Clone, Copy, ValueEnum, Default)]
pub enum OutputFormat {
    /// 文本格式（默认）
    #[default]
    Text,
    /// JSON 格式
    Json,
    /// CSV 格式
    Csv,
}

/// Benchmark 命令参数
pub struct BenchmarkArgs {
    /// 测试套件
    pub suite: BenchmarkSuite,
    /// 输出格式
    pub output: OutputFormat,
    /// 迭代次数
    pub iterations: usize,
    /// 测试数据路径
    pub data_path: Option<String>,
    /// 预热次数
    pub warmup: usize,
}

/// 单个基准测试结果
#[derive(Debug, Clone)]
struct BenchmarkResult {
    name: String,
    duration_ms: f64,
    throughput: Option<String>,
    status: BenchmarkStatus,
}

#[derive(Debug, Clone)]
enum BenchmarkStatus {
    Success,
    Pending,
    Error(String),
}

/// 获取当前使用的分配器名称
fn allocator_name() -> &'static str {
    if cfg!(feature = "mimalloc") {
        "mimalloc"
    } else {
        "system"
    }
}

/// 执行基准测试命令
pub fn execute(args: BenchmarkArgs) -> Result<()> {
    let path = args.data_path.clone().unwrap_or_else(|| ".".to_string());

    println!("{}\n", format!("XORE 性能基准测试 (分配器: {})", allocator_name()).cyan().bold());
    println!(
        "测试路径: {}, 迭代次数: {}, 预热: {}\n",
        path.yellow(),
        args.iterations.to_string().cyan(),
        args.warmup.to_string().cyan()
    );

    let mut results = Vec::new();

    match args.suite {
        BenchmarkSuite::All => {
            results.extend(run_scan_benchmark(&path, args.iterations, args.warmup)?);
            results.extend(run_io_benchmark(&path, args.iterations, args.warmup)?);
            results.extend(run_alloc_benchmark(args.iterations)?);
            results.extend(run_search_benchmark()?);
            results.extend(run_process_benchmark()?);
        }
        BenchmarkSuite::Scan => {
            results.extend(run_scan_benchmark(&path, args.iterations, args.warmup)?);
        }
        BenchmarkSuite::Search => {
            results.extend(run_search_benchmark()?);
        }
        BenchmarkSuite::Process => {
            results.extend(run_process_benchmark()?);
        }
        BenchmarkSuite::Io => {
            results.extend(run_io_benchmark(&path, args.iterations, args.warmup)?);
        }
        BenchmarkSuite::Alloc => {
            results.extend(run_alloc_benchmark(args.iterations)?);
        }
    }

    // 输出结果
    match args.output {
        OutputFormat::Text => print_text_results(&results),
        OutputFormat::Json => print_json_results(&results)?,
        OutputFormat::Csv => print_csv_results(&results),
    }

    Ok(())
}

/// 运行文件扫描基准测试
fn run_scan_benchmark(
    path: &str,
    iterations: usize,
    warmup: usize,
) -> Result<Vec<BenchmarkResult>> {
    let mut results = Vec::new();

    // 预热
    for _ in 0..warmup {
        let config = ScanConfig::new(path);
        let scanner = FileScanner::new(config);
        let _ = scanner.scan();
    }

    // 正式测试
    let mut durations = Vec::with_capacity(iterations);
    let mut total_files = 0u64;
    let mut total_dirs = 0u64;

    for _ in 0..iterations {
        let config = ScanConfig::new(path);
        let scanner = FileScanner::new(config);
        let start = Instant::now();
        let (files, stats) = scanner.scan()?;
        let elapsed = start.elapsed();

        durations.push(elapsed);
        total_files = stats.total_files as u64;
        total_dirs = stats.directories as u64;

        drop(files);
    }

    let avg_duration = average_duration(&durations);
    let files_per_sec = if avg_duration.as_secs_f64() > 0.0 {
        (total_files as f64 / avg_duration.as_secs_f64()) as u64
    } else {
        0
    };

    results.push(BenchmarkResult {
        name: format!("文件扫描 ({} 文件, {} 目录)", total_files, total_dirs),
        duration_ms: avg_duration.as_secs_f64() * 1000.0,
        throughput: Some(format!("{} files/s", format_number(files_per_sec))),
        status: BenchmarkStatus::Success,
    });

    // 深度遍历测试
    let config = ScanConfig::new(path).with_max_depth(10);
    let scanner = FileScanner::new(config);
    let start = Instant::now();
    let _ = scanner.scan()?;
    let elapsed = start.elapsed();

    results.push(BenchmarkResult {
        name: "目录遍历 (深度 10)".to_string(),
        duration_ms: elapsed.as_secs_f64() * 1000.0,
        throughput: None,
        status: BenchmarkStatus::Success,
    });

    Ok(results)
}

/// 运行 I/O 基准测试
fn run_io_benchmark(path: &str, iterations: usize, warmup: usize) -> Result<Vec<BenchmarkResult>> {
    let mut results = Vec::new();

    // 找一个测试文件
    let config = ScanConfig::new(path).with_max_depth(3);
    let scanner = FileScanner::new(config);
    let (files, _) = scanner.scan()?;

    // 找一个适合测试的文件（1KB - 100MB）
    let test_file =
        files.iter().find(|f| f.size > 1024 && f.size < 100 * 1024 * 1024).map(|f| f.path.clone());

    if let Some(file_path) = test_file {
        let file_size = std::fs::metadata(&file_path)?.len();

        // 预热
        for _ in 0..warmup {
            let mut file = std::fs::File::open(&file_path)?;
            let mut buffer = Vec::new();
            let _ = file.read_to_end(&mut buffer);
        }

        // 顺序读取测试
        let mut durations = Vec::with_capacity(iterations);

        for _ in 0..iterations {
            let mut file = std::fs::File::open(&file_path)?;
            let mut buffer = Vec::new();
            let start = Instant::now();
            file.read_to_end(&mut buffer)?;
            durations.push(start.elapsed());
        }

        let avg_duration = average_duration(&durations);
        let bytes_per_sec = if avg_duration.as_secs_f64() > 0.0 {
            (file_size as f64 / avg_duration.as_secs_f64()) as u64
        } else {
            0
        };

        results.push(BenchmarkResult {
            name: format!("顺序读取 ({})", format_bytes(file_size)),
            duration_ms: avg_duration.as_secs_f64() * 1000.0,
            throughput: Some(format!("{}/s", format_bytes(bytes_per_sec))),
            status: BenchmarkStatus::Success,
        });

        // 写入测试（临时文件）
        let temp_path = std::env::temp_dir().join("xore_benchmark_test");
        let test_data = vec![0u8; 1024 * 1024]; // 1MB

        let mut write_durations = Vec::with_capacity(iterations);
        for _ in 0..iterations {
            let start = Instant::now();
            let mut file = std::fs::File::create(&temp_path)?;
            file.write_all(&test_data)?;
            file.sync_all()?;
            write_durations.push(start.elapsed());
        }

        let _ = std::fs::remove_file(&temp_path);

        let avg_write = average_duration(&write_durations);
        let write_speed = if avg_write.as_secs_f64() > 0.0 {
            (1024.0 * 1024.0 / avg_write.as_secs_f64()) as u64
        } else {
            0
        };

        results.push(BenchmarkResult {
            name: "顺序写入 (1 MB)".to_string(),
            duration_ms: avg_write.as_secs_f64() * 1000.0,
            throughput: Some(format!("{}/s", format_bytes(write_speed))),
            status: BenchmarkStatus::Success,
        });
    } else {
        results.push(BenchmarkResult {
            name: "I/O 测试".to_string(),
            duration_ms: 0.0,
            throughput: None,
            status: BenchmarkStatus::Error("未找到合适的测试文件".to_string()),
        });
    }

    Ok(results)
}

/// 运行搜索基准测试（待实现）
fn run_search_benchmark() -> Result<Vec<BenchmarkResult>> {
    Ok(vec![BenchmarkResult {
        name: "全文搜索".to_string(),
        duration_ms: 0.0,
        throughput: None,
        status: BenchmarkStatus::Pending,
    }])
}

/// 运行数据处理基准测试（待实现）
fn run_process_benchmark() -> Result<Vec<BenchmarkResult>> {
    Ok(vec![BenchmarkResult {
        name: "数据处理".to_string(),
        duration_ms: 0.0,
        throughput: None,
        status: BenchmarkStatus::Pending,
    }])
}

/// 运行内存分配基准测试
fn run_alloc_benchmark(iterations: usize) -> Result<Vec<BenchmarkResult>> {
    let mut results = Vec::new();

    // Vec<String> 分配测试
    let mut durations = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let start = Instant::now();
        let mut v: Vec<String> = Vec::with_capacity(100_000);
        for i in 0..100_000 {
            v.push(format!("path/to/file_{}.txt", i));
        }
        drop(v);
        durations.push(start.elapsed());
    }

    let avg = average_duration(&durations);
    let allocs_per_sec =
        if avg.as_secs_f64() > 0.0 { (100_000.0 / avg.as_secs_f64()) as u64 } else { 0 };

    results.push(BenchmarkResult {
        name: "Vec<String> 分配 (100K 元素)".to_string(),
        duration_ms: avg.as_secs_f64() * 1000.0,
        throughput: Some(format!("{} allocs/s", format_number(allocs_per_sec))),
        status: BenchmarkStatus::Success,
    });

    // HashMap 分配测试
    let mut durations = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let start = Instant::now();
        let mut map: std::collections::HashMap<String, usize> =
            std::collections::HashMap::with_capacity(50_000);
        for i in 0..50_000 {
            map.insert(format!("key_{}", i), i);
        }
        drop(map);
        durations.push(start.elapsed());
    }

    let avg = average_duration(&durations);
    let ops_per_sec =
        if avg.as_secs_f64() > 0.0 { (50_000.0 / avg.as_secs_f64()) as u64 } else { 0 };

    results.push(BenchmarkResult {
        name: "HashMap<String, usize> (50K 条目)".to_string(),
        duration_ms: avg.as_secs_f64() * 1000.0,
        throughput: Some(format!("{} ops/s", format_number(ops_per_sec))),
        status: BenchmarkStatus::Success,
    });

    // 小字符串频繁分配释放测试
    let mut durations = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let start = Instant::now();
        for i in 0..50_000 {
            let s = format!("temp_string_{}", i);
            std::hint::black_box(&s);
        }
        durations.push(start.elapsed());
    }

    let avg = average_duration(&durations);
    let allocs_per_sec =
        if avg.as_secs_f64() > 0.0 { (50_000.0 / avg.as_secs_f64()) as u64 } else { 0 };

    results.push(BenchmarkResult {
        name: "小字符串分配/释放 (50K 次)".to_string(),
        duration_ms: avg.as_secs_f64() * 1000.0,
        throughput: Some(format!("{} allocs/s", format_number(allocs_per_sec))),
        status: BenchmarkStatus::Success,
    });

    Ok(results)
}

/// 计算平均耗时
fn average_duration(durations: &[Duration]) -> Duration {
    if durations.is_empty() {
        return Duration::ZERO;
    }
    let total: Duration = durations.iter().sum();
    total / durations.len() as u32
}

/// 格式化数字（添加千分位分隔符）
fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.insert(0, ',');
        }
        result.insert(0, c);
    }
    result
}

/// 格式化字节大小
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// 打印文本格式结果
fn print_text_results(results: &[BenchmarkResult]) {
    println!("{}", "测试结果".bold());
    println!("{}", "─".repeat(60));

    for result in results {
        let (icon, status_color) = match &result.status {
            BenchmarkStatus::Success => (ICON_SUCCESS, "green"),
            BenchmarkStatus::Pending => (ICON_PENDING, "yellow"),
            BenchmarkStatus::Error(_) => ("✗", "red"),
        };

        let icon_colored = match status_color {
            "green" => icon.green().to_string(),
            "yellow" => icon.yellow().to_string(),
            "red" => icon.red().to_string(),
            _ => icon.to_string(),
        };

        match &result.status {
            BenchmarkStatus::Success => {
                let duration = format!("{:.1}ms", result.duration_ms);
                let throughput = result
                    .throughput
                    .as_ref()
                    .map(|t| format!(" ({})", t.cyan()))
                    .unwrap_or_default();

                println!("{} {}: {}{}", icon_colored, result.name, duration.yellow(), throughput);
            }
            BenchmarkStatus::Pending => {
                println!("{} {}: {}", icon_colored, result.name, "待实现".dimmed());
            }
            BenchmarkStatus::Error(msg) => {
                println!("{} {}: {}", icon_colored, result.name, msg.red());
            }
        }
    }

    println!();
}

/// 打印 JSON 格式结果
fn print_json_results(results: &[BenchmarkResult]) -> Result<()> {
    let json_results: Vec<serde_json::Value> = results
        .iter()
        .map(|r| {
            serde_json::json!({
                "name": r.name,
                "duration_ms": r.duration_ms,
                "throughput": r.throughput,
                "status": match &r.status {
                    BenchmarkStatus::Success => "success",
                    BenchmarkStatus::Pending => "pending",
                    BenchmarkStatus::Error(_) => "error",
                }
            })
        })
        .collect();

    let output = serde_json::to_string_pretty(&json_results)?;
    println!("{}", output);
    Ok(())
}

/// 打印 CSV 格式结果
fn print_csv_results(results: &[BenchmarkResult]) {
    println!("name,duration_ms,throughput,status");
    for result in results {
        let status = match &result.status {
            BenchmarkStatus::Success => "success",
            BenchmarkStatus::Pending => "pending",
            BenchmarkStatus::Error(_) => "error",
        };
        println!(
            "\"{}\",{:.2},{},{}",
            result.name,
            result.duration_ms,
            result.throughput.as_deref().unwrap_or(""),
            status
        );
    }
}
