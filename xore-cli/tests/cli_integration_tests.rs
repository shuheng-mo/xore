//! CLI 集成测试
//!
//! 测试 XORE CLI 的各个命令是否正常工作。

use std::process::Command;

/// 获取 cargo 二进制路径
fn cargo_bin() -> Command {
    Command::new(env!("CARGO"))
}

#[test]
fn test_cli_help() {
    let output = cargo_bin()
        .args(["run", "--package", "xore-cli", "--", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "xore --help should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("xore"), "Help should mention 'xore'");
    assert!(stdout.contains("find"), "Help should list 'find' command");
    assert!(stdout.contains("process"), "Help should list 'process' command");
    assert!(stdout.contains("benchmark"), "Help should list 'benchmark' command");
}

#[test]
fn test_cli_version() {
    let output = cargo_bin()
        .args(["run", "--package", "xore-cli", "--", "--version"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "xore --version should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("1.0.0"), "Version should be 1.0.0");
}

#[test]
fn test_cli_find_help() {
    let output = cargo_bin()
        .args(["run", "--package", "xore-cli", "--", "find", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "xore find --help should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--path"), "Find help should mention --path");
    assert!(stdout.contains("--type"), "Find help should mention --type");
    assert!(stdout.contains("--size"), "Find help should mention --size");
    assert!(stdout.contains("--mtime"), "Find help should mention --mtime");
}

#[test]
fn test_cli_process_help() {
    let output = cargo_bin()
        .args(["run", "--package", "xore-cli", "--", "process", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "xore process --help should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--quality-check"), "Process help should mention --quality-check");
}

#[test]
fn test_cli_benchmark_help() {
    let output = cargo_bin()
        .args(["run", "--package", "xore-cli", "--", "benchmark", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "xore benchmark --help should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--suite"), "Benchmark help should mention --suite");
    assert!(stdout.contains("alloc"), "Benchmark help should mention 'alloc' suite");
    assert!(stdout.contains("--iterations"), "Benchmark help should mention --iterations");
}

#[test]
fn test_cli_find_basic() {
    let output = cargo_bin()
        .args(["run", "--package", "xore-cli", "--", "find", "--path", ".", "--max-depth", "1"])
        .output()
        .expect("Failed to execute command");

    // 命令应该成功执行
    assert!(output.status.success(), "xore find should succeed");
}

#[test]
fn test_cli_find_with_type_filter() {
    let output = cargo_bin()
        .args([
            "run",
            "--package",
            "xore-cli",
            "--",
            "find",
            "--path",
            ".",
            "--type",
            "code",
            "--max-depth",
            "2",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "xore find with type filter should succeed");
}

#[test]
fn test_cli_benchmark_scan() {
    let output = cargo_bin()
        .args([
            "run",
            "--package",
            "xore-cli",
            "--",
            "benchmark",
            "--suite",
            "scan",
            "-n",
            "1",
            "--warmup",
            "0",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "xore benchmark scan should succeed");
}

#[test]
fn test_cli_benchmark_alloc() {
    let output = cargo_bin()
        .args(["run", "--package", "xore-cli", "--", "benchmark", "--suite", "alloc", "-n", "1"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "xore benchmark alloc should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Vec<String>") || stdout.contains("分配"),
        "Alloc benchmark should output allocation results"
    );
}

#[test]
fn test_cli_verbose_mode() {
    let output = cargo_bin()
        .args([
            "run",
            "--package",
            "xore-cli",
            "--",
            "--verbose",
            "find",
            "--path",
            ".",
            "--max-depth",
            "1",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "xore --verbose find should succeed");
}

#[test]
fn test_cli_quiet_mode() {
    let output = cargo_bin()
        .args([
            "run",
            "--package",
            "xore-cli",
            "--",
            "--quiet",
            "find",
            "--path",
            ".",
            "--max-depth",
            "1",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "xore --quiet find should succeed");
}

#[test]
fn test_cli_alias_f() {
    let output = cargo_bin()
        .args(["run", "--package", "xore-cli", "--", "f", "--path", ".", "--max-depth", "1"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "xore f (alias for find) should succeed");
}

#[test]
fn test_cli_alias_bench() {
    let output = cargo_bin()
        .args([
            "run",
            "--package",
            "xore-cli",
            "--",
            "bench",
            "--suite",
            "scan",
            "-n",
            "1",
            "--warmup",
            "0",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "xore bench (alias for benchmark) should succeed");
}
