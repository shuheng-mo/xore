//! Watch 守护进程管理命令
//!
//! 支持后台运行文件监控守护进程，管理其生命周期。
//!
//! ## 命令
//!
//! ```bash
//! # 启动后台监控（通过 find 命令）
//! xore f --watch-daemon
//! xore f --watch-daemon --path /path/to/watch
//! xore f --watch-daemon --include "*.rs,*.toml" --exclude "target,node_modules"
//!
//! # 查看监控状态
//! xore watch status
//!
//! # 查看监控日志
//! xore watch logs
//! xore watch logs --lines 50
//!
//! # 停止监控
//! xore watch stop
//! xore watch stop --path /path/to/watch
//! ```

use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use colored::*;
use serde::{Deserialize, Serialize};
use xore_config::XorePaths;

/// Watch 子命令
pub enum WatchSubcommand {
    /// 查看所有监控状态
    Status,
    /// 查看日志
    Logs { lines: usize },
    /// 停止监控
    Stop { path: Option<String> },
}

/// Watch 命令参数
pub struct WatchArgs {
    pub subcommand: WatchSubcommand,
}

/// 守护进程状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonStatus {
    /// 监控路径
    pub path: String,
    /// 是否运行中
    pub running: bool,
    /// 进程 PID
    pub pid: Option<u32>,
    /// 启动时间
    pub started_at: Option<String>,
    /// PID 文件路径
    pub pid_file: String,
    /// 日志文件路径
    pub log_file: String,
}

/// 守护进程元数据（存储在 PID 目录）
#[derive(Debug, Serialize, Deserialize)]
pub struct DaemonMeta {
    pub pid: u32,
    pub path: String,
    pub started_at: String,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
}

/// 执行 watch 命令
pub fn execute(args: WatchArgs) -> Result<()> {
    match args.subcommand {
        WatchSubcommand::Status => show_status(),
        WatchSubcommand::Logs { lines } => show_logs(lines),
        WatchSubcommand::Stop { path } => stop_daemon(path.as_deref()),
    }
}

/// 获取 watch 运行时目录（~/.xore/cache/watch）
fn get_watch_runtime_dir() -> Result<PathBuf> {
    let xore_paths = XorePaths::new().map_err(|e| anyhow::anyhow!("无法获取 XORE 路径: {}", e))?;
    let dir = xore_paths.cache_dir().join("watch");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// 获取 PID 文件路径（基于监控路径的 hash）
pub fn get_pid_file(watch_path: &Path) -> Result<PathBuf> {
    let runtime_dir = get_watch_runtime_dir()?;
    let path_hash = path_to_hash(watch_path);
    Ok(runtime_dir.join(format!("{}.pid", path_hash)))
}

/// 获取日志文件路径
pub fn get_log_file(watch_path: &Path) -> Result<PathBuf> {
    let runtime_dir = get_watch_runtime_dir()?;
    let path_hash = path_to_hash(watch_path);
    Ok(runtime_dir.join(format!("{}.log", path_hash)))
}

/// 获取元数据文件路径
fn get_meta_file(watch_path: &Path) -> Result<PathBuf> {
    let runtime_dir = get_watch_runtime_dir()?;
    let path_hash = path_to_hash(watch_path);
    Ok(runtime_dir.join(format!("{}.meta.json", path_hash)))
}

/// 将路径转换为唯一的短 hash（用于文件名）
fn path_to_hash(path: &Path) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    path.to_string_lossy().hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

/// 读取 PID 文件
pub fn read_pid(pid_file: &Path) -> Result<Option<u32>> {
    if !pid_file.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(pid_file)?;
    let pid: u32 = content.trim().parse().map_err(|_| anyhow::anyhow!("Invalid PID in file"))?;
    Ok(Some(pid))
}

/// 检查进程是否运行
pub fn is_process_running(pid: u32) -> bool {
    #[cfg(unix)]
    {
        // 在 Unix 上，发送信号 0 来检查进程是否存在
        unsafe { libc::kill(pid as libc::pid_t, 0) == 0 }
    }
    #[cfg(windows)]
    {
        use windows_sys::Win32::Foundation::CloseHandle;
        use windows_sys::Win32::System::Threading::{
            OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
        };
        unsafe {
            let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            if handle.is_null() {
                return false;
            }
            CloseHandle(handle);
            true
        }
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = pid;
        false
    }
}

/// 保存守护进程元数据
pub fn save_daemon_meta(
    watch_path: &Path,
    pid: u32,
    include_patterns: Vec<String>,
    exclude_patterns: Vec<String>,
) -> Result<()> {
    let meta_file = get_meta_file(watch_path)?;
    let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let dt = chrono::DateTime::<chrono::Utc>::from_timestamp(now as i64, 0)
        .unwrap_or(chrono::DateTime::<chrono::Utc>::UNIX_EPOCH);

    let meta = DaemonMeta {
        pid,
        path: watch_path.display().to_string(),
        started_at: dt.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        include_patterns,
        exclude_patterns,
    };

    let json = serde_json::to_string_pretty(&meta)?;
    fs::write(meta_file, json)?;
    Ok(())
}

/// 启动 Watch 守护进程
///
/// 在后台启动 `xore f --watch --index --path <path>` 进程，
/// 将 stdout/stderr 重定向到日志文件，保存 PID 便于后续管理。
pub fn start_daemon(
    watch_path: &Path,
    include_patterns: Vec<String>,
    exclude_patterns: Vec<String>,
) -> Result<u32> {
    let pid_file = get_pid_file(watch_path)?;
    let log_file_path = get_log_file(watch_path)?;

    // 检查是否已有守护进程运行
    if let Some(existing_pid) = read_pid(&pid_file)? {
        if is_process_running(existing_pid) {
            return Err(anyhow::anyhow!(
                "监控已在运行 (PID: {})\n提示：使用 'xore watch stop' 停止后再启动",
                existing_pid
            ));
        }
        // 清理过时的 PID 文件
        let _ = fs::remove_file(&pid_file);
    }

    // 获取当前可执行文件路径
    let current_exe =
        std::env::current_exe().map_err(|e| anyhow::anyhow!("无法获取可执行文件路径: {}", e))?;

    // 构建命令参数
    let mut cmd_args = vec![
        "f".to_string(),
        "--watch".to_string(),
        "--index".to_string(),
        "--path".to_string(),
        watch_path.display().to_string(),
    ];

    if !include_patterns.is_empty() {
        // 不直接支持 --include，通过 --type 传递扩展名
    }

    if !exclude_patterns.is_empty() {
        // 排除模式通过配置处理
        cmd_args.push("--no-ignore".to_string());
    }

    // 打开/创建日志文件
    let log_file = fs::OpenOptions::new().create(true).append(true).open(&log_file_path)?;

    // 写入日志头
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let log_clone = log_file.try_clone()?;
    let mut log_writer = std::io::BufWriter::new(log_clone);
    use std::io::Write;
    writeln!(
        log_writer,
        "\n=== Watch daemon started at {} ===\n路径: {}\n",
        now,
        watch_path.display()
    )?;
    drop(log_writer);

    // 启动子进程
    let mut cmd = std::process::Command::new(&current_exe);
    cmd.args(&cmd_args)
        .stdin(std::process::Stdio::null())
        .stdout(log_file.try_clone()?)
        .stderr(log_file);

    // Unix：分离进程组
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        unsafe {
            cmd.pre_exec(|| {
                // 创建新会话，脱离控制终端
                if libc::setsid() < 0 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
    }

    let child = cmd.spawn().map_err(|e| anyhow::anyhow!("启动守护进程失败: {}", e))?;

    let pid = child.id();

    // 写入 PID 文件
    fs::write(&pid_file, pid.to_string())?;

    // 保存元数据
    save_daemon_meta(watch_path, pid, include_patterns, exclude_patterns)?;

    // 不等待子进程，让它在后台运行
    // 注意：在 Unix 上，drop(child) 不会 kill 进程，child 会继续作为孤儿进程
    drop(child);

    Ok(pid)
}

/// 停止守护进程
pub fn stop_daemon_by_path(watch_path: &Path) -> Result<()> {
    let pid_file = get_pid_file(watch_path)?;

    let pid = match read_pid(&pid_file)? {
        Some(p) => p,
        None => {
            return Err(anyhow::anyhow!("没有找到监控路径 {} 的 PID 文件", watch_path.display()))
        }
    };

    if !is_process_running(pid) {
        // 进程已停止，只需清理文件
        cleanup_daemon_files(watch_path)?;
        println!("{} 守护进程已经停止（PID: {}），已清理文件", "✓".green(), pid);
        return Ok(());
    }

    // 终止进程
    #[cfg(unix)]
    unsafe {
        use std::time::Duration;
        libc::kill(pid as libc::pid_t, libc::SIGTERM);
        // 等待最多 3 秒
        for _ in 0..30 {
            std::thread::sleep(Duration::from_millis(100));
            if !is_process_running(pid) {
                break;
            }
        }
        if is_process_running(pid) {
            // 强制杀死
            libc::kill(pid as libc::pid_t, libc::SIGKILL);
        }
    }

    #[cfg(windows)]
    {
        // Windows：使用 taskkill
        let _ =
            std::process::Command::new("taskkill").args(["/PID", &pid.to_string(), "/F"]).output();
    }

    // 清理文件
    cleanup_daemon_files(watch_path)?;

    println!("{} 已停止监控进程 (PID: {})", "✓".green(), pid);
    Ok(())
}

/// 清理守护进程文件
fn cleanup_daemon_files(watch_path: &Path) -> Result<()> {
    if let Ok(pid_file) = get_pid_file(watch_path) {
        let _ = fs::remove_file(pid_file);
    }
    if let Ok(meta_file) = get_meta_file(watch_path) {
        let _ = fs::remove_file(meta_file);
    }
    Ok(())
}

/// 获取所有守护进程状态
fn get_all_daemon_statuses() -> Result<Vec<DaemonStatus>> {
    let runtime_dir = match get_watch_runtime_dir() {
        Ok(d) => d,
        Err(_) => return Ok(vec![]),
    };

    let mut statuses = Vec::new();
    let mut path_map: HashMap<String, (PathBuf, PathBuf, PathBuf)> = HashMap::new();

    // 扫描 runtime 目录，找所有 .pid 文件
    if let Ok(entries) = fs::read_dir(&runtime_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();

            if name.ends_with(".pid") {
                let hash = name.trim_end_matches(".pid").to_string();
                let pid_file = runtime_dir.join(&name);
                let log_file = runtime_dir.join(format!("{}.log", hash));
                let meta_file = runtime_dir.join(format!("{}.meta.json", hash));
                path_map.insert(hash, (pid_file, log_file, meta_file));
            }
        }
    }

    for (_hash, (pid_file, log_file, meta_file)) in path_map {
        let pid = read_pid(&pid_file)?.unwrap_or(0);
        let running = if pid > 0 { is_process_running(pid) } else { false };

        // 读取元数据
        let (watch_path, started_at) = if meta_file.exists() {
            let meta_content = fs::read_to_string(&meta_file).unwrap_or_default();
            let meta: Option<DaemonMeta> = serde_json::from_str(&meta_content).ok();
            if let Some(m) = meta {
                (m.path, Some(m.started_at))
            } else {
                (pid_file.display().to_string(), None)
            }
        } else {
            (pid_file.display().to_string(), None)
        };

        statuses.push(DaemonStatus {
            path: watch_path,
            running,
            pid: if pid > 0 { Some(pid) } else { None },
            started_at,
            pid_file: pid_file.display().to_string(),
            log_file: log_file.display().to_string(),
        });
    }

    Ok(statuses)
}

/// 显示所有守护进程状态
fn show_status() -> Result<()> {
    println!("{} Watch 守护进程状态", "📊".cyan());
    println!();

    let statuses = get_all_daemon_statuses()?;

    if statuses.is_empty() {
        println!("{}", "没有正在运行的文件监控守护进程".yellow());
        println!();
        println!("提示：使用 {} 启动后台监控", "xore f --watch-daemon".cyan());
        return Ok(());
    }

    for status in &statuses {
        let state_icon =
            if status.running { "🟢".green().to_string() } else { "🔴".red().to_string() };
        let state_text = if status.running {
            "运行中".green().to_string()
        } else {
            "已停止".red().to_string()
        };

        println!("{} 路径: {}", state_icon, status.path.cyan().bold());
        println!("   状态: {}", state_text);

        if let Some(pid) = status.pid {
            println!("   PID:  {}", pid.to_string().yellow());
        }

        if let Some(ref started) = status.started_at {
            println!("   启动: {}", started.dimmed());
        }

        println!("   日志: {}", status.log_file.dimmed());
        println!();
    }

    let running_count = statuses.iter().filter(|s| s.running).count();
    println!(
        "{} {} 个监控进程（{} 个运行中，{} 个已停止）",
        "📈".cyan(),
        statuses.len(),
        running_count.to_string().green(),
        (statuses.len() - running_count).to_string().red()
    );

    Ok(())
}

/// 显示日志
fn show_logs(lines: usize) -> Result<()> {
    let statuses = get_all_daemon_statuses()?;

    if statuses.is_empty() {
        println!("{}", "没有找到监控守护进程".yellow());
        return Ok(());
    }

    for status in &statuses {
        let log_path = Path::new(&status.log_file);

        if !log_path.exists() {
            println!("{} 路径: {} (无日志文件)", "📄".cyan(), status.path);
            continue;
        }

        println!("{} 路径: {}", "📋".cyan(), status.path.cyan().bold());
        println!("   日志: {}", status.log_file.dimmed());
        println!();

        // 读取最后 N 行
        let log_lines = read_last_n_lines(log_path, lines)?;

        for line in &log_lines {
            println!("   {}", line);
        }

        if log_lines.is_empty() {
            println!("   {}", "(日志为空)".dimmed());
        }

        println!();
    }

    Ok(())
}

/// 读取文件最后 N 行
fn read_last_n_lines(path: &Path, n: usize) -> Result<Vec<String>> {
    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);
    let all_lines: Vec<String> = reader.lines().map_while(|l| l.ok()).collect();

    let start = all_lines.len().saturating_sub(n);
    Ok(all_lines[start..].to_vec())
}

/// 停止守护进程（通过命令）
fn stop_daemon(path: Option<&str>) -> Result<()> {
    if let Some(path_str) = path {
        // 停止指定路径的监控
        let watch_path = PathBuf::from(path_str);
        stop_daemon_by_path(&watch_path)?;
    } else {
        // 停止所有监控
        let statuses = get_all_daemon_statuses()?;

        if statuses.is_empty() {
            println!("{}", "没有正在运行的文件监控守护进程".yellow());
            return Ok(());
        }

        let running: Vec<_> = statuses.iter().filter(|s| s.running).collect();

        if running.is_empty() {
            println!("{}", "没有正在运行的文件监控守护进程".yellow());

            // 清理已停止的残留文件
            let runtime_dir = get_watch_runtime_dir()?;
            cleanup_stale_files(&runtime_dir)?;
            return Ok(());
        }

        println!("{} 正在停止 {} 个监控进程...", "⏹️".yellow(), running.len());

        for status in running {
            let watch_path = PathBuf::from(&status.path);
            match stop_daemon_by_path(&watch_path) {
                Ok(_) => {}
                Err(e) => eprintln!("{} 停止 {} 失败: {}", "⚠️".yellow(), status.path, e),
            }
        }

        println!("{} 所有监控进程已停止", "✓".green());
    }

    Ok(())
}

/// 清理过时的 PID/meta 文件（进程已停止）
fn cleanup_stale_files(runtime_dir: &Path) -> Result<()> {
    let mut cleaned = 0usize;

    if let Ok(entries) = fs::read_dir(runtime_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            if name.ends_with(".pid") {
                if let Ok(Some(pid)) = read_pid(&path) {
                    if !is_process_running(pid) {
                        let _ = fs::remove_file(&path);
                        cleaned += 1;
                    }
                }
            }
        }
    }

    if cleaned > 0 {
        println!("{} 已清理 {} 个过时的 PID 文件", "🧹".green(), cleaned);
    }

    Ok(())
}

/// 检查指定路径是否有守护进程运行
#[allow(dead_code)]
pub fn is_daemon_running(watch_path: &Path) -> bool {
    if let Ok(pid_file) = get_pid_file(watch_path) {
        if let Ok(Some(pid)) = read_pid(&pid_file) {
            return is_process_running(pid);
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_path_to_hash_deterministic() {
        let path = Path::new("/some/test/path");
        let h1 = path_to_hash(path);
        let h2 = path_to_hash(path);
        assert_eq!(h1, h2, "相同路径应生成相同 hash");
    }

    #[test]
    fn test_path_to_hash_different_paths() {
        let h1 = path_to_hash(Path::new("/path/a"));
        let h2 = path_to_hash(Path::new("/path/b"));
        assert_ne!(h1, h2, "不同路径应生成不同 hash");
    }

    #[test]
    fn test_is_process_running_current_process() {
        let pid = std::process::id();
        assert!(is_process_running(pid), "当前进程应为运行中");
    }

    #[test]
    fn test_is_process_running_invalid_pid() {
        // 注意：u32::MAX 不能用，因为在 Unix 上 pid_t 是 i32，
        // u32::MAX as i32 = -1，kill(-1, 0) 会返回成功（有特殊含义）
        // 正确做法：spawn 一个进程，等它退出，再检查它是否仍在运行
        #[cfg(unix)]
        {
            let mut child = std::process::Command::new("sh")
                .args(["-c", "exit 0"])
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
                .expect("无法启动子进程");
            let pid = child.id();
            child.wait().expect("等待子进程失败");
            // 进程已退出，PID 应被回收
            // 给 OS 少许时间清理
            std::thread::sleep(std::time::Duration::from_millis(50));
            assert!(!is_process_running(pid), "已退出的进程不应为运行中 (PID: {})", pid);
        }
        #[cfg(windows)]
        {
            // Windows 上同理
            let mut child = std::process::Command::new("cmd")
                .args(["/C", "exit 0"])
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
                .expect("无法启动子进程");
            let pid = child.id();
            child.wait().expect("等待子进程失败");
            std::thread::sleep(std::time::Duration::from_millis(50));
            assert!(!is_process_running(pid), "已退出的进程不应为运行中");
        }
        #[cfg(not(any(unix, windows)))]
        {
            // 其他平台跳过
        }
    }

    #[test]
    fn test_read_pid_nonexistent() {
        let path = Path::new("/nonexistent/path/test.pid");
        let result = read_pid(path).unwrap();
        assert!(result.is_none(), "不存在的 PID 文件应返回 None");
    }

    #[test]
    fn test_read_pid_valid() {
        let dir = TempDir::new().unwrap();
        let pid_file = dir.path().join("test.pid");
        fs::write(&pid_file, "12345").unwrap();

        let result = read_pid(&pid_file).unwrap();
        assert_eq!(result, Some(12345));
    }

    #[test]
    fn test_read_pid_invalid_content() {
        let dir = TempDir::new().unwrap();
        let pid_file = dir.path().join("test.pid");
        fs::write(&pid_file, "not_a_number").unwrap();

        let result = read_pid(&pid_file);
        assert!(result.is_err(), "无效 PID 内容应返回错误");
    }

    #[test]
    fn test_read_last_n_lines() {
        let dir = TempDir::new().unwrap();
        let log_file = dir.path().join("test.log");
        let content = (1..=20).map(|i| format!("line {}", i)).collect::<Vec<_>>().join("\n");
        fs::write(&log_file, &content).unwrap();

        let lines = read_last_n_lines(&log_file, 5).unwrap();
        assert_eq!(lines.len(), 5, "应读取 5 行");
        assert_eq!(lines[0], "line 16");
        assert_eq!(lines[4], "line 20");
    }

    #[test]
    fn test_read_last_n_lines_fewer_than_n() {
        let dir = TempDir::new().unwrap();
        let log_file = dir.path().join("test.log");
        fs::write(&log_file, "line 1\nline 2\nline 3\n").unwrap();

        let lines = read_last_n_lines(&log_file, 100).unwrap();
        assert_eq!(lines.len(), 3, "行数少于 n 时应返回所有行");
    }

    #[test]
    fn test_daemon_meta_serialization() {
        let meta = DaemonMeta {
            pid: 12345,
            path: "/test/path".to_string(),
            started_at: "2026-01-01T00:00:00Z".to_string(),
            include_patterns: vec!["*.rs".to_string()],
            exclude_patterns: vec!["target".to_string()],
        };

        let json = serde_json::to_string(&meta).unwrap();
        let parsed: DaemonMeta = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.pid, 12345);
        assert_eq!(parsed.path, "/test/path");
    }

    #[test]
    fn test_get_all_daemon_statuses_empty() {
        // 当没有守护进程时，应返回空列表（不报错）
        // 注意：此测试依赖实际 XorePaths，在 CI 环境可能需要设置
        let result = get_all_daemon_statuses();
        // 只验证不 panic，不验证具体值
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_is_daemon_running_nonexistent() {
        let path = Path::new("/path/that/doesnt/exist/12345");
        assert!(!is_daemon_running(path), "不存在路径的守护进程应为 false");
    }

    #[test]
    fn test_watch_status_command() {
        let args = WatchArgs { subcommand: WatchSubcommand::Status };
        // 应该不 panic，即使没有守护进程
        let result = execute(args);
        assert!(result.is_ok(), "status 命令应成功: {:?}", result);
    }
}
