//! XORE 输出模块 - Token 节省可视化
//!
//! 提供 Token 消耗计算和节省信息显示功能。

use std::sync::atomic::{AtomicU64, Ordering};

/// 全局累计节省统计
static TOTAL_SAVINGS: AtomicU64 = AtomicU64::new(0);

/// Token 计算常量
pub mod token_constants {
    /// 传统方式估算：每字节消耗的 Token 数（约 0.5 token/字节）
    pub const TRADITIONAL_TOKEN_PER_BYTE: f64 = 0.5;

    /// GPT-4o 价格（每百万 token）
    pub const GPT4O_PRICE_PER_MILLION: f64 = 3.0;
    /// GPT-4o mini 价格（每百万 token）
    pub const GPT4O_MINI_PRICE_PER_MILLION: f64 = 0.15;

    /// 人民币汇率（1 USD = 7.2 CNY）
    pub const CNY_EXCHANGE_RATE: f64 = 7.2;
}

/// Token 节省信息
#[derive(Debug, Clone)]
pub struct TokenSavings {
    /// 传统方式估算的 Token 消耗
    pub traditional_tokens: u64,
    /// XORE 实际消耗的 Token
    pub actual_tokens: u64,
    /// 节省的 Token 数量
    pub saved_tokens: u64,
    /// 节省的美元金额
    pub saved_usd: f64,
    /// 节省的人民币金额
    pub saved_cny: f64,
}

impl TokenSavings {
    /// 计算节省信息
    ///
    /// # Arguments
    /// * `file_size` - 文件大小（字节）
    /// * `output_length` - XORE 输出内容长度（字符数）
    pub fn calculate(file_size: u64, output_length: usize) -> Self {
        // 传统方式估算：文件大小 × 0.5 token/字节
        let traditional_tokens =
            ((file_size as f64) * token_constants::TRADITIONAL_TOKEN_PER_BYTE) as u64;

        // XORE 实际消耗：输出内容 × 0.5 token/字符
        let actual_tokens =
            ((output_length as f64) * token_constants::TRADITIONAL_TOKEN_PER_BYTE) as u64;

        // 节省的 Token
        let saved_tokens = traditional_tokens.saturating_sub(actual_tokens);

        // 计算费用（使用 GPT-4o mini 价格，更实惠）
        let saved_usd =
            (saved_tokens as f64) / 1_000_000.0 * token_constants::GPT4O_MINI_PRICE_PER_MILLION;
        let saved_cny = saved_usd * token_constants::CNY_EXCHANGE_RATE;

        Self { traditional_tokens, actual_tokens, saved_tokens, saved_usd, saved_cny }
    }

    /// 格式化输出（极简模式）
    pub fn format_minimal(&self) -> String {
        if self.saved_tokens == 0 {
            return String::new();
        }

        if self.saved_cny < 0.01 {
            format!("省 {} Token", self.saved_tokens)
        } else {
            format!("省 {} Token ≈ ¥{:.2}", self.saved_tokens, self.saved_cny)
        }
    }

    /// 格式化输出（详细模式）
    pub fn format_detailed(&self) -> String {
        if self.saved_tokens == 0 {
            return String::new();
        }

        format!(
            "💸 节省对比：传统读全文需 {} Token，xore仅用 {} Token，节省 {} Token ≈ ¥{:.2}",
            self.traditional_tokens, self.actual_tokens, self.saved_tokens, self.saved_cny
        )
    }

    /// 格式化输出（累计模式）
    pub fn format_cumulative(&self) -> String {
        if self.saved_tokens == 0 {
            return String::new();
        }

        let total = TOTAL_SAVINGS.load(Ordering::Relaxed);
        let total_cny = (total as f64) / 1_000_000.0
            * token_constants::GPT4O_MINI_PRICE_PER_MILLION
            * token_constants::CNY_EXCHANGE_RATE;

        format!("📊 累计为您节省：{} Token ≈ ¥{:.2}", total, total_cny)
    }

    /// 添加到累计统计
    pub fn add_to_total(&self) {
        TOTAL_SAVINGS.fetch_add(self.saved_tokens, Ordering::Relaxed);
    }
}

/// 输出模式
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputMode {
    /// 极简模式
    Minimal,
    /// 详细模式
    Detailed,
    /// 累计模式
    Cumulative,
}

impl OutputMode {
    /// 从配置字符串创建
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s {
            "detailed" => OutputMode::Detailed,
            "cumulative" => OutputMode::Cumulative,
            _ => OutputMode::Minimal,
        }
    }
}

/// 输出格式化器
pub struct OutputFormatter {
    /// 是否显示节省信息
    pub show_savings: bool,
    /// 输出模式
    pub mode: OutputMode,
    /// 货币单位
    pub currency: String,
}

impl OutputFormatter {
    /// 从配置创建
    pub fn from_config(show_savings: bool, savings_mode: &str, currency: &str) -> Self {
        Self {
            show_savings,
            mode: OutputMode::from_str(savings_mode),
            currency: currency.to_string(),
        }
    }

    /// 格式化并输出节省信息
    pub fn print_savings(&self, savings: &TokenSavings) {
        if !self.show_savings || savings.saved_tokens == 0 {
            return;
        }

        let output = match self.mode {
            OutputMode::Minimal => savings.format_minimal(),
            OutputMode::Detailed => savings.format_detailed(),
            OutputMode::Cumulative => {
                // 先添加到累计
                savings.add_to_total();
                // 显示本次节省 + 累计
                format!(
                    "{}\n📊 累计为您节省：{} Token ≈ ¥{:.2}",
                    savings.format_minimal(),
                    TOTAL_SAVINGS.load(Ordering::Relaxed),
                    (TOTAL_SAVINGS.load(Ordering::Relaxed) as f64) / 1_000_000.0
                        * token_constants::GPT4O_MINI_PRICE_PER_MILLION
                        * token_constants::CNY_EXCHANGE_RATE
                )
            }
        };

        if !output.is_empty() {
            println!("\n{}", output);
        }
    }
}

/// 重置累计统计（用于测试）
pub fn reset_total_savings() {
    TOTAL_SAVINGS.store(0, Ordering::Relaxed);
}

/// 获取当前累计节省
pub fn get_total_savings() -> u64 {
    TOTAL_SAVINGS.load(Ordering::Relaxed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_savings_calculation() {
        // 模拟：文件 1000 字节，输出 100 字符
        let savings = TokenSavings::calculate(1000, 100);

        // 传统：1000 * 0.5 = 500 token
        assert_eq!(savings.traditional_tokens, 500);
        // 实际：100 * 0.5 = 50 token
        assert_eq!(savings.actual_tokens, 50);
        // 节省：500 - 50 = 450 token
        assert_eq!(savings.saved_tokens, 450);
    }

    #[test]
    fn test_format_minimal() {
        let savings = TokenSavings::calculate(1000, 100);
        let output = savings.format_minimal();

        assert!(output.contains("450"));
        assert!(output.contains("Token"));
    }

    #[test]
    fn test_format_detailed() {
        let savings = TokenSavings::calculate(1000, 100);
        let output = savings.format_detailed();

        assert!(output.contains("500"));
        assert!(output.contains("50"));
        assert!(output.contains("450"));
    }

    #[test]
    fn test_output_mode_from_str() {
        assert_eq!(OutputMode::from_str("minimal"), OutputMode::Minimal);
        assert_eq!(OutputMode::from_str("detailed"), OutputMode::Detailed);
        assert_eq!(OutputMode::from_str("cumulative"), OutputMode::Cumulative);
    }

    #[test]
    fn test_zero_savings() {
        // 当节省为 0 时，格式化输出应为空
        let savings = TokenSavings::calculate(0, 0);
        assert_eq!(savings.saved_tokens, 0);
        assert!(savings.format_minimal().is_empty());
        assert!(savings.format_detailed().is_empty());
    }
}
