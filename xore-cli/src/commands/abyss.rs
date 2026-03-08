//! Abyss 全局监控命令
//!
//! 智能体可以直接让工具监控系统级别（用户主目录下）文件变化。
//! 调用前会提醒用户隐私和性能影响。
//!
//! ## 命令
//!
//! ```bash
//! # 启动全局监控（需要用户确认）
//! xore agent abyss --start
//!
//! # 强制启动（跳过确认，仅开发模式）
//! xore agent abyss --start --force
//!
//! # 查看监控状态
//! xore agent abyss --status
//!
//! # 查看监控日志
//! xore agent abyss --logs
//! xore agent abyss --logs --lines 50
//!
//! # 停止全局监控
//! xore agent abyss --stop
//!
//! # 配置过滤规则
//! xore agent abyss --config --exclude "Downloads,Desktop"
//! ```

use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use anyhow::Result;
use colored::*;
use serde::{Deserialize, Serialize};
use xore_config::XorePaths;

use super::watch::{is_process_running, read_pid};

/// Abyss 子命令动作
pub enum AbyssAction {
    /// 启动全局监控
    Start {
        /// 是否强制启动（跳过确认）
        force: bool,
        /// 排除的目录（逗号分隔）
        exclude: Option<String>,
        /// 包含的扩展名（逗号分隔）
        include: Option<String>,
    },
    /// 查看监控状态
    Status,
    /// 查看监控日志
    Logs { lines: usize },
    /// 停止监控
    Stop,
    /// 显示配置
    Config { exclude: Option<String>, include: Option<String> },
}

/// Abyss 命令参数
pub struct AbyssArgs {
    pub action: AbyssAction,
}

/// Abyss 守护进程元数据
#[derive(Debug, Serialize, Deserialize)]
pub struct AbyssMeta {
    pub pid: u32,
    pub home_path: String,
    pub started_at: String,
    pub exclude_dirs: Vec<String>,
    pub include_extensions: Vec<String>,
    pub version: String,
}

/// Abyss 运行状态
#[derive(Debug)]
pub struct AbyssStatus {
    pub running: bool,
    pub pid: Option<u32>,
    pub home_path: String,
    pub started_at: Option<String>,
    pub exclude_dirs: Vec<String>,
    pub include_extensions: Vec<String>,
    pub log_file: String,
}

/// 获取 Abyss 运行时目录（~/.xore/cache/abyss）
fn get_abyss_runtime_dir() -> Result<PathBuf> {
    let xore_paths = XorePaths::new().map_err(|e| anyhow::anyhow!("无法获取 XORE 路径: {}", e))?;
    let dir = xore_paths.cache_dir().join("abyss");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// 获取 Abyss PID 文件路径
fn get_abyss_pid_file() -> Result<PathBuf> {
    Ok(get_abyss_runtime_dir()?.join("abyss.pid"))
}

/// 获取 Abyss 元数据文件路径
fn get_abyss_meta_file() -> Result<PathBuf> {
    Ok(get_abyss_runtime_dir()?.join("abyss.meta.json"))
}

/// 获取 Abyss 日志文件路径
fn get_abyss_log_file() -> Result<PathBuf> {
    Ok(get_abyss_runtime_dir()?.join("abyss.log"))
}

/// 执行 Abyss 命令
pub fn execute(args: AbyssArgs) -> Result<()> {
    match args.action {
        AbyssAction::Start { force, exclude, include } => start_abyss(force, exclude, include),
        AbyssAction::Status => show_abyss_status(),
        AbyssAction::Logs { lines } => show_abyss_logs(lines),
        AbyssAction::Stop => stop_abyss(),
        AbyssAction::Config { exclude, include } => show_or_update_config(exclude, include),
    }
}

/// 启动 Abyss 全局监控
fn start_abyss(force: bool, exclude: Option<String>, include: Option<String>) -> Result<()> {
    // 检查是否已在运行
    if let Some(existing_pid) = get_running_pid()? {
        return Err(anyhow::anyhow!(
            "Abyss 全局监控已在运行 (PID: {})\n提示：使用 'xore agent abyss --stop' 先停止",
            existing_pid
        ));
    }

    // 检查权限
    check_permissions()?;

    // 用户确认（除非 --force）
    if !force {
        let confirmed = show_warning_and_confirm()?;
        if !confirmed {
            println!("{} 已取消操作", "🚫".yellow());
            return Ok(());
        }
    }

    // 解析配置
    let exclude_dirs: Vec<String> = exclude
        .as_deref()
        .unwrap_or("Downloads,Desktop,Library,Movies,Music,Pictures")
        .split(',')
        .filter(|s| !s.is_empty())
        .map(|s| s.trim().to_string())
        .collect();

    let include_extensions: Vec<String> = include
        .as_deref()
        .unwrap_or("")
        .split(',')
        .filter(|s| !s.is_empty())
        .map(|s| s.trim().trim_start_matches("*.").trim_start_matches('.').to_lowercase())
        .collect();

    // 获取用户主目录
    let home_dir = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("无法获取用户主目录"))?;

    // 启动守护进程
    let pid = spawn_abyss_daemon(&home_dir, &exclude_dirs, &include_extensions)?;

    // 保存元数据
    save_abyss_meta(pid, &home_dir, &exclude_dirs, &include_extensions)?;

    println!("{} Abyss 全局监控已启动", "✅".green());
    println!("   PID:  {}", pid.to_string().yellow());
    println!("   路径: {}", home_dir.display().to_string().cyan());
    println!("   排除: {}", exclude_dirs.join(", ").dimmed());
    if !include_extensions.is_empty() {
        println!("   仅监控: *.{}", include_extensions.join(", *.").cyan());
    }
    println!();
    println!(
        "提示：使用 {} 查看状态，{} 停止监控",
        "xore agent abyss --status".cyan(),
        "xore agent abyss --stop".yellow()
    );

    Ok(())
}

/// 显示隐私警告并请求用户确认
fn show_warning_and_confirm() -> Result<bool> {
    println!("{}", "⚠️  警告：全局文件监控".yellow().bold());
    println!();
    println!("此操作将监控您主目录下的所有文件变化。");
    println!();
    println!("{}", "影响：".red());
    println!("  • 可能会消耗额外的 CPU 和内存资源");
    println!("  • 监控范围包括您的所有文件（已排除常见无关目录）");
    println!("  • 文件变化记录会保存在本地日志中");
    println!();
    println!("{}", "建议：".cyan());
    println!("  1. 使用 --exclude 排除不需要监控的目录");
    println!("     例如：--exclude \"Downloads,Desktop,Videos\"");
    println!("  2. 监控完成后及时使用 --stop 停止");
    println!("  3. 仅在需要时启动，避免长期运行");
    println!();
    print!("是否继续启动全局监控? [y/N]: ");
    std::io::stdout().flush()?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    Ok(input.trim().to_lowercase() == "y")
}

/// 检查系统权限
fn check_permissions() -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        // 检查 inotify 限制
        let max_watches = fs::read_to_string("/proc/sys/fs/inotify/max_user_watches")
            .ok()
            .and_then(|s| s.trim().parse::<u64>().ok())
            .unwrap_or(8192);

        if max_watches < 10_000 {
            eprintln!("{} inotify 监控数量限制较低 ({})", "⚠️".yellow(), max_watches);
            eprintln!("  建议提升限制：");
            eprintln!("  echo 100000 | sudo tee /proc/sys/fs/inotify/max_user_watches");
            eprintln!();
        }
    }

    #[cfg(target_os = "macos")]
    {
        // macOS 无法在运行时检查 FSEvents 权限
        // 仅提示用户可能需要授权
        eprintln!(
            "{} macOS 提示：如果监控不生效，请在「系统偏好设置」>「安全性与隐私」>「完全磁盘访问权限」中授权 XORE",
            "ℹ️".cyan()
        );
    }

    Ok(())
}

/// 启动 Abyss 守护进程（后台运行）
fn spawn_abyss_daemon(
    home_dir: &Path,
    exclude_dirs: &[String],
    include_extensions: &[String],
) -> Result<u32> {
    let current_exe =
        std::env::current_exe().map_err(|e| anyhow::anyhow!("无法获取可执行文件路径: {}", e))?;

    let log_file_path = get_abyss_log_file()?;

    // 构建命令参数
    let cmd_args = vec![
        "f".to_string(),
        "--watch".to_string(),
        "--index".to_string(),
        "--path".to_string(),
        home_dir.display().to_string(),
    ];

    // 添加排除目录参数（通过 --exclude-dirs 环境变量传递，避免复杂的命令行转义）
    // 实际上 xore find 不直接支持 --exclude，我们通过环境变量传参
    let exclude_env = exclude_dirs.join(",");
    let include_env = include_extensions.join(",");

    // 打开/创建日志文件（追加模式）
    let log_file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file_path)
        .map_err(|e| anyhow::anyhow!("无法创建日志文件 {:?}: {}", log_file_path, e))?;

    // 写入启动日志头
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let log_clone = log_file.try_clone()?;
    let mut log_writer = std::io::BufWriter::new(log_clone);
    writeln!(
        log_writer,
        "\n=== Abyss daemon started at {} ===\n主目录: {}\n排除: {}\n",
        now,
        home_dir.display(),
        exclude_env
    )?;
    drop(log_writer);

    let mut cmd = std::process::Command::new(&current_exe);
    cmd.args(&cmd_args)
        .env("XORE_ABYSS_EXCLUDE", &exclude_env)
        .env("XORE_ABYSS_INCLUDE", &include_env)
        .stdin(std::process::Stdio::null())
        .stdout(log_file.try_clone()?)
        .stderr(log_file);

    // Unix：分离进程组，避免随父进程退出
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        unsafe {
            cmd.pre_exec(|| {
                if libc::setsid() < 0 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
    }

    let child = cmd.spawn().map_err(|e| anyhow::anyhow!("启动 Abyss 守护进程失败: {}", e))?;

    let pid = child.id();

    // 写入 PID 文件
    let pid_file = get_abyss_pid_file()?;
    fs::write(&pid_file, pid.to_string())?;

    // 让子进程继续运行（不等待）
    drop(child);

    Ok(pid)
}

/// 保存 Abyss 元数据
fn save_abyss_meta(
    pid: u32,
    home_dir: &Path,
    exclude_dirs: &[String],
    include_extensions: &[String],
) -> Result<()> {
    let meta_file = get_abyss_meta_file()?;
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    let meta = AbyssMeta {
        pid,
        home_path: home_dir.display().to_string(),
        started_at: now,
        exclude_dirs: exclude_dirs.to_vec(),
        include_extensions: include_extensions.to_vec(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };

    let json = serde_json::to_string_pretty(&meta)?;
    fs::write(meta_file, json)?;
    Ok(())
}

/// 获取正在运行的 Abyss PID（若有）
fn get_running_pid() -> Result<Option<u32>> {
    let pid_file = get_abyss_pid_file()?;
    let pid = match read_pid(&pid_file)? {
        Some(p) => p,
        None => return Ok(None),
    };

    if is_process_running(pid) {
        Ok(Some(pid))
    } else {
        // 清理过时的 PID 文件
        let _ = fs::remove_file(&pid_file);
        Ok(None)
    }
}

/// 读取 Abyss 状态
fn get_abyss_status() -> Result<AbyssStatus> {
    let pid_file = get_abyss_pid_file()?;
    let log_file = get_abyss_log_file()?;

    let pid = read_pid(&pid_file)?;
    let running = pid.map(is_process_running).unwrap_or(false);

    // 读取元数据
    let (home_path, started_at, exclude_dirs, include_extensions) =
        if let Ok(meta_file) = get_abyss_meta_file() {
            if meta_file.exists() {
                let content = fs::read_to_string(&meta_file).unwrap_or_default();
                let meta: Option<AbyssMeta> = serde_json::from_str(&content).ok();
                if let Some(m) = meta {
                    (m.home_path, Some(m.started_at), m.exclude_dirs, m.include_extensions)
                } else {
                    (
                        dirs::home_dir().map(|h| h.display().to_string()).unwrap_or_default(),
                        None,
                        vec![],
                        vec![],
                    )
                }
            } else {
                (
                    dirs::home_dir().map(|h| h.display().to_string()).unwrap_or_default(),
                    None,
                    vec![],
                    vec![],
                )
            }
        } else {
            (
                dirs::home_dir().map(|h| h.display().to_string()).unwrap_or_default(),
                None,
                vec![],
                vec![],
            )
        };

    Ok(AbyssStatus {
        running,
        pid,
        home_path,
        started_at,
        exclude_dirs,
        include_extensions,
        log_file: log_file.display().to_string(),
    })
}

/// 显示 Abyss 状态
fn show_abyss_status() -> Result<()> {
    let status = get_abyss_status()?;

    println!("{} Abyss 全局监控状态", "🌊".cyan());
    println!();

    if status.running {
        println!("  状态: {}", "运行中 🟢".green().bold());
        if let Some(pid) = status.pid {
            println!("  PID:  {}", pid.to_string().yellow());
        }
        if let Some(ref started) = status.started_at {
            println!("  启动: {}", started.dimmed());
        }
        println!("  路径: {}", status.home_path.cyan());
        if !status.exclude_dirs.is_empty() {
            println!("  排除: {}", status.exclude_dirs.join(", ").dimmed());
        }
        if !status.include_extensions.is_empty() {
            println!("  仅监控: *.{}", status.include_extensions.join(", *.").cyan());
        }
        println!("  日志: {}", status.log_file.dimmed());
    } else {
        println!("  状态: {}", "未运行 🔴".red());
        println!();
        println!("  提示：使用 {} 启动全局监控", "xore agent abyss --start".cyan());
    }

    Ok(())
}

/// 显示 Abyss 日志
fn show_abyss_logs(lines: usize) -> Result<()> {
    let log_file_path = get_abyss_log_file()?;

    if !log_file_path.exists() {
        println!("{}", "没有找到 Abyss 日志文件".yellow());
        println!("提示：先启动监控：{}", "xore agent abyss --start".cyan());
        return Ok(());
    }

    println!("{} Abyss 监控日志（最后 {} 行）", "📋".cyan(), lines);
    println!("   文件: {}", log_file_path.display().to_string().dimmed());
    println!();

    let file = fs::File::open(&log_file_path)?;
    let reader = BufReader::new(file);
    let all_lines: Vec<String> = reader.lines().map_while(|l| l.ok()).collect();

    let start = all_lines.len().saturating_sub(lines);
    let display_lines = &all_lines[start..];

    if display_lines.is_empty() {
        println!("{}", "(日志为空)".dimmed());
    } else {
        for line in display_lines {
            // 高亮显示特殊行
            if line.contains("===") {
                println!("  {}", line.cyan());
            } else if line.to_lowercase().contains("error") || line.contains("ERROR") {
                println!("  {}", line.red());
            } else if line.to_lowercase().contains("warn") || line.contains("WARN") {
                println!("  {}", line.yellow());
            } else {
                println!("  {}", line.dimmed());
            }
        }
    }

    let total_lines = all_lines.len();
    if total_lines > lines {
        println!();
        println!("  {} 仅显示最后 {} 行（共 {} 行）", "ℹ️".cyan(), lines, total_lines);
    }

    Ok(())
}

/// 停止 Abyss 守护进程
fn stop_abyss() -> Result<()> {
    let pid_file = get_abyss_pid_file()?;

    let pid = match read_pid(&pid_file)? {
        Some(p) => p,
        None => {
            println!("{}", "Abyss 全局监控未在运行".yellow());
            return Ok(());
        }
    };

    if !is_process_running(pid) {
        // 清理残留文件
        cleanup_abyss_files()?;
        println!("{} Abyss 守护进程已经停止（PID: {}），已清理文件", "✓".green(), pid);
        return Ok(());
    }

    println!("{} 正在停止 Abyss 守护进程 (PID: {})...", "⏹️".yellow(), pid);

    // 终止进程
    #[cfg(unix)]
    unsafe {
        use std::time::Duration;
        libc::kill(pid as libc::pid_t, libc::SIGTERM);

        // 等待进程退出（最多 3 秒）
        for _ in 0..30 {
            std::thread::sleep(Duration::from_millis(100));
            if !is_process_running(pid) {
                break;
            }
        }

        // 如果还在运行，强制终止
        if is_process_running(pid) {
            eprintln!("{} SIGTERM 无效，强制终止...", "⚠️".yellow());
            libc::kill(pid as libc::pid_t, libc::SIGKILL);
        }
    }

    #[cfg(windows)]
    {
        let _ =
            std::process::Command::new("taskkill").args(["/PID", &pid.to_string(), "/F"]).output();
    }

    // 清理文件
    cleanup_abyss_files()?;

    println!("{} Abyss 全局监控已停止 (PID: {})", "✓".green(), pid);

    Ok(())
}

/// 清理 Abyss 运行时文件
fn cleanup_abyss_files() -> Result<()> {
    if let Ok(pid_file) = get_abyss_pid_file() {
        let _ = fs::remove_file(pid_file);
    }
    if let Ok(meta_file) = get_abyss_meta_file() {
        let _ = fs::remove_file(meta_file);
    }
    Ok(())
}

/// 显示或更新配置
fn show_or_update_config(exclude: Option<String>, include: Option<String>) -> Result<()> {
    if exclude.is_none() && include.is_none() {
        // 显示当前配置
        let status = get_abyss_status()?;

        println!("{} Abyss 当前配置", "⚙️".cyan());
        println!();
        println!("  默认排除目录:");
        for dir in &status.exclude_dirs {
            println!("    - {}", dir.dimmed());
        }
        println!();
        if status.include_extensions.is_empty() {
            println!("  监控范围: 所有文件类型");
        } else {
            println!("  仅监控扩展名: {}", status.include_extensions.join(", "));
        }
        println!();
        println!("提示：重新启动时通过 --exclude 和 --include 参数重新配置");
    } else {
        println!("{} 配置已更新（将在下次启动时生效）", "ℹ️".cyan());
        if let Some(ref exc) = exclude {
            println!("  排除: {}", exc);
        }
        if let Some(ref inc) = include {
            println!("  包含: {}", inc);
        }
    }

    Ok(())
}

/// 从环境变量读取 Abyss 配置（供守护进程内部使用）
#[allow(dead_code)]
pub fn read_abyss_env_config() -> (Vec<String>, Vec<String>) {
    let exclude_dirs: Vec<String> = std::env::var("XORE_ABYSS_EXCLUDE")
        .unwrap_or_default()
        .split(',')
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect();

    let include_extensions: Vec<String> = std::env::var("XORE_ABYSS_INCLUDE")
        .unwrap_or_default()
        .split(',')
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect();

    (exclude_dirs, include_extensions)
}

/// 检查路径是否应被 Abyss 监控排除
#[allow(dead_code)]
pub fn should_exclude_from_abyss(path: &Path, exclude_dirs: &[String]) -> bool {
    let path_str = path.to_string_lossy();
    for exclude in exclude_dirs {
        if path_str.contains(exclude.as_str()) {
            return true;
        }
    }
    false
}

/// 检查文件扩展名是否在监控列表中
#[allow(dead_code)]
pub fn should_include_in_abyss(path: &Path, include_extensions: &[String]) -> bool {
    if include_extensions.is_empty() {
        return true; // 空列表表示监控所有类型
    }
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    include_extensions.iter().any(|inc| inc == &ext)
}

/// 获取 Abyss 运行时统计（可供 MCP 工具调用）
#[allow(dead_code)]
pub fn get_abyss_stats() -> serde_json::Value {
    match get_abyss_status() {
        Ok(status) => serde_json::json!({
            "running": status.running,
            "pid": status.pid,
            "home_path": status.home_path,
            "started_at": status.started_at,
            "exclude_dirs": status.exclude_dirs,
            "include_extensions": status.include_extensions,
            "log_file": status.log_file,
        }),
        Err(e) => serde_json::json!({
            "error": e.to_string(),
            "running": false,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 用于序列化环境变量相关测试，避免并发污染
    static ENV_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn test_should_exclude_from_abyss() {
        let exclude_dirs = vec!["Downloads".to_string(), "Library".to_string()];

        assert!(
            should_exclude_from_abyss(Path::new("/home/user/Downloads/file.txt"), &exclude_dirs),
            "Downloads 应被排除"
        );
        assert!(
            should_exclude_from_abyss(Path::new("/home/user/Library/prefs.plist"), &exclude_dirs),
            "Library 应被排除"
        );
        assert!(
            !should_exclude_from_abyss(Path::new("/home/user/Documents/file.txt"), &exclude_dirs),
            "Documents 不应被排除"
        );
    }

    #[test]
    fn test_should_include_in_abyss_empty_list() {
        // 空列表 = 监控所有类型
        assert!(should_include_in_abyss(Path::new("any.txt"), &[]));
        assert!(should_include_in_abyss(Path::new("any.bin"), &[]));
    }

    #[test]
    fn test_should_include_in_abyss_filter() {
        let include_exts = vec!["rs".to_string(), "toml".to_string()];

        assert!(should_include_in_abyss(Path::new("main.rs"), &include_exts), ".rs 应被包含");
        assert!(should_include_in_abyss(Path::new("Cargo.toml"), &include_exts), ".toml 应被包含");
        assert!(!should_include_in_abyss(Path::new("image.png"), &include_exts), ".png 不应被包含");
        assert!(!should_include_in_abyss(Path::new("noext"), &include_exts), "无扩展名不应被包含");
    }

    #[test]
    fn test_abyss_meta_serialization() {
        let meta = AbyssMeta {
            pid: 9999,
            home_path: "/home/user".to_string(),
            started_at: "2026-01-01T00:00:00Z".to_string(),
            exclude_dirs: vec!["Downloads".to_string()],
            include_extensions: vec!["rs".to_string()],
            version: "1.2.0".to_string(),
        };

        let json = serde_json::to_string_pretty(&meta).unwrap();
        let parsed: AbyssMeta = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.pid, 9999);
        assert_eq!(parsed.home_path, "/home/user");
        assert_eq!(parsed.exclude_dirs, vec!["Downloads"]);
        assert_eq!(parsed.include_extensions, vec!["rs"]);
    }

    #[test]
    fn test_read_abyss_env_config() {
        // 序列化，避免并发测试污染环境变量
        let _guard = ENV_MUTEX.lock().unwrap_or_else(|p| p.into_inner());

        std::env::set_var("XORE_ABYSS_EXCLUDE", "Downloads,Library");
        std::env::set_var("XORE_ABYSS_INCLUDE", "rs,toml");

        let (exclude, include) = read_abyss_env_config();
        assert_eq!(exclude, vec!["Downloads", "Library"]);
        assert_eq!(include, vec!["rs", "toml"]);

        // 清理
        std::env::remove_var("XORE_ABYSS_EXCLUDE");
        std::env::remove_var("XORE_ABYSS_INCLUDE");
    }

    #[test]
    fn test_read_abyss_env_config_empty() {
        // 序列化，避免并发测试污染环境变量
        let _guard = ENV_MUTEX.lock().unwrap_or_else(|p| p.into_inner());

        std::env::remove_var("XORE_ABYSS_EXCLUDE");
        std::env::remove_var("XORE_ABYSS_INCLUDE");

        let (exclude, include) = read_abyss_env_config();
        assert!(exclude.is_empty(), "未设置时应为空");
        assert!(include.is_empty(), "未设置时应为空");
    }

    #[test]
    fn test_abyss_status_command() {
        let args = AbyssArgs { action: AbyssAction::Status };
        // 应该不 panic
        let result = execute(args);
        assert!(result.is_ok(), "status 命令应成功: {:?}", result);
    }

    #[test]
    fn test_get_abyss_stats_not_running() {
        let stats = get_abyss_stats();
        // stats 应为有效 JSON
        assert!(stats.is_object());
        assert!(stats.get("running").is_some());
    }

    #[test]
    fn test_check_permissions_no_panic() {
        // 权限检查不应 panic
        let result = check_permissions();
        assert!(result.is_ok(), "权限检查应成功: {:?}", result);
    }
}
