//! Config 命令实现
//!
//! 提供全局配置管理功能，允许用户查看和修改 ~/.xore/config.toml 配置文件。

use anyhow::{Context, Result};
use colored::*;
use std::fs;
use xore_config::{Config, XorePaths};

/// Config 命令参数
pub struct ConfigArgs {
    pub subcommand: ConfigSubcommand,
}

#[derive(Debug, Clone)]
pub enum ConfigSubcommand {
    Show,
    Get { key: String },
    Set { key: String, value: String },
    Reset,
    Edit,
}

/// 执行配置命令
pub fn execute(args: ConfigArgs) -> Result<()> {
    match args.subcommand {
        ConfigSubcommand::Show => show_config()?,
        ConfigSubcommand::Get { key } => get_config_value(&key)?,
        ConfigSubcommand::Set { key, value } => set_config_value(&key, &value)?,
        ConfigSubcommand::Reset => reset_config()?,
        ConfigSubcommand::Edit => edit_config()?,
    }
    Ok(())
}

/// 显示当前配置
fn show_config() -> Result<()> {
    let xore_paths = XorePaths::new().context("无法获取 XORE 路径")?;
    let config_file = xore_paths.config_file();

    if !config_file.exists() {
        println!("{}", "配置文件不存在，正在创建默认配置...".yellow());
        let config = Config::default();
        config.save(&config_file).context("无法保存默认配置")?;
        println!("默认配置已创建于: {}", config_file.display());
    }

    let config = Config::load(&config_file).context("无法加载配置文件")?;
    let toml_string = toml::to_string_pretty(&config).context("无法序列化配置")?;

    println!("{}", "=== XORE 全局配置 ===".cyan());
    println!();
    println!("配置文件路径: {}", config_file.display());
    println!();
    println!("{}", toml_string);

    Ok(())
}

/// 获取配置项的值
fn get_config_value(key: &str) -> Result<()> {
    let xore_paths = XorePaths::new().context("无法获取 XORE 路径")?;
    let config_file = xore_paths.config_file();

    if !config_file.exists() {
        anyhow::bail!("配置文件不存在，请先运行 'xore config show' 创建配置");
    }

    let config = Config::load(&config_file).context("无法加载配置文件")?;

    // 解析嵌套的键（例如：paths.index）
    let parts: Vec<&str> = key.split('.').collect();

    if parts.is_empty() {
        anyhow::bail!("无效的配置键");
    }

    // 使用 serde_json 来遍历嵌套结构
    let json_value = serde_json::to_value(&config).context("无法序列化配置")?;

    // 遍历嵌套键
    let mut current = &json_value;
    for part in parts {
        current = current.get(part).context(format!("配置键 '{}' 不存在", key))?;
    }

    println!("{} = {}", key, current);
    Ok(())
}

/// 设置配置项的值
fn set_config_value(key: &str, value: &str) -> Result<()> {
    let xore_paths = XorePaths::new().context("无法获取 XORE 路径")?;
    let config_file = xore_paths.config_file();

    // 确保配置目录存在
    if let Some(parent) = config_file.parent() {
        fs::create_dir_all(parent).context("无法创建配置目录")?;
    }

    // 加载现有配置或创建默认配置
    let config = if config_file.exists() {
        Config::load(&config_file).context("无法加载配置文件")?
    } else {
        Config::default()
    };

    // 解析并设置值
    let parts: Vec<&str> = key.split('.').collect();

    if parts.is_empty() {
        anyhow::bail!("无效的配置键");
    }

    // 使用 serde_json 来处理嵌套结构
    let mut json_value = serde_json::to_value(&config).context("无法序列化配置")?;

    // 遍历到倒数第二层
    let mut current: &mut serde_json::Value = &mut json_value;
    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            // 最后一层，设置值
            // 尝试解析为数字，否则使用字符串
            let parsed_value = if let Ok(num) = value.parse::<i64>() {
                serde_json::Value::Number(num.into())
            } else if let Ok(float) = value.parse::<f64>() {
                serde_json::Number::from_f64(float)
                    .map(serde_json::Value::Number)
                    .unwrap_or_else(|| serde_json::Value::String(value.to_string()))
            } else if value == "true" {
                serde_json::Value::Bool(true)
            } else if value == "false" {
                serde_json::Value::Bool(false)
            } else {
                serde_json::Value::String(value.to_string())
            };

            if let Some(map) = current.as_object_mut() {
                map.insert(part.to_string(), parsed_value);
            } else {
                anyhow::bail!("无法设置配置值：路径无效");
            }
        } else {
            current = current.get_mut(part).context(format!("配置键路径 '{}' 不存在", key))?;
        }
    }

    println!("{}", format!("设置 {} = {}", key, value).green());
    println!("配置文件路径: {}", config_file.display());

    // 将修改后的 JSON Value 反序列化回 Config，再用 TOML 格式写入
    let updated_config: Config =
        serde_json::from_value(json_value).context("无法将修改后的值反序列化为配置结构")?;
    updated_config.save(&config_file).context("无法保存配置文件")?;

    println!("{}", "配置已更新".green());
    Ok(())
}

/// 重置配置为默认值
fn reset_config() -> Result<()> {
    let xore_paths = XorePaths::new().context("无法获取 XORE 路径")?;
    let config_file = xore_paths.config_file();

    if config_file.exists() {
        // 备份现有配置
        let backup_file = config_file.with_extension("toml.bak");
        fs::copy(&config_file, &backup_file).context("无法备份配置")?;
        println!("{}", format!("已备份现有配置到: {}", backup_file.display()).yellow());
    }

    let config = Config::default();
    config.save(&config_file).context("无法保存默认配置")?;

    println!("{}", "配置已重置为默认值".green());
    Ok(())
}

/// 编辑配置文件
fn edit_config() -> Result<()> {
    let xore_paths = XorePaths::new().context("无法获取 XORE 路径")?;
    let config_file = xore_paths.config_file();

    // 确保配置存在
    if !config_file.exists() {
        let config = Config::default();
        config.save(&config_file).context("无法创建默认配置")?;
        println!("{}", "已创建默认配置文件".yellow());
    }

    // 使用默认编辑器打开配置文件
    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| "vi".to_string());

    println!("{}", format!("使用 {} 编辑配置文件...", editor).cyan());
    println!("配置文件路径: {}", config_file.display());

    std::process::Command::new(&editor).arg(&config_file).status().context("无法打开编辑器")?;

    println!("{}", "配置文件已更新".green());
    Ok(())
}
