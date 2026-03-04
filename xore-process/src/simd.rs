//! SIMD 优化模块 - 高性能数值计算
//!
//! 提供优化的数值计算函数，用于提升数据处理性能。
//! 注意：由于 std::simd 仍不稳定，这里使用手动优化和循环展开技术。

/// 优化的 f64 数组求和
///
/// 使用循环展开技术提升性能
pub fn sum_f64_simd(data: &[f64]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }

    // 使用 4 路循环展开
    let mut sum0 = 0.0;
    let mut sum1 = 0.0;
    let mut sum2 = 0.0;
    let mut sum3 = 0.0;

    let chunks = data.chunks_exact(4);
    let remainder = chunks.remainder();

    for chunk in chunks {
        sum0 += chunk[0];
        sum1 += chunk[1];
        sum2 += chunk[2];
        sum3 += chunk[3];
    }

    let mut sum = sum0 + sum1 + sum2 + sum3;

    // 处理剩余元素
    for &val in remainder {
        sum += val;
    }

    sum
}

/// 优化的 f64 数组均值计算
pub fn mean_f64_simd(data: &[f64]) -> Option<f64> {
    if data.is_empty() {
        return None;
    }

    let sum = sum_f64_simd(data);
    Some(sum / data.len() as f64)
}

/// 优化的 f64 数组方差计算
///
/// 使用两次遍历算法：
/// 1. 计算均值
/// 2. 计算平方差之和
pub fn variance_f64_simd(data: &[f64]) -> Option<f64> {
    if data.len() < 2 {
        return None;
    }

    let mean = mean_f64_simd(data)?;

    // 使用 4 路循环展开计算平方差
    let mut sum0 = 0.0;
    let mut sum1 = 0.0;
    let mut sum2 = 0.0;
    let mut sum3 = 0.0;

    let chunks = data.chunks_exact(4);
    let remainder = chunks.remainder();

    for chunk in chunks {
        let diff0 = chunk[0] - mean;
        let diff1 = chunk[1] - mean;
        let diff2 = chunk[2] - mean;
        let diff3 = chunk[3] - mean;

        sum0 += diff0 * diff0;
        sum1 += diff1 * diff1;
        sum2 += diff2 * diff2;
        sum3 += diff3 * diff3;
    }

    let mut variance = sum0 + sum1 + sum2 + sum3;

    // 处理剩余元素
    for &val in remainder {
        let diff = val - mean;
        variance += diff * diff;
    }

    Some(variance / (data.len() - 1) as f64)
}

/// 优化的 f64 数组标准差计算
pub fn std_dev_f64_simd(data: &[f64]) -> Option<f64> {
    variance_f64_simd(data).map(|v| v.sqrt())
}

/// 优化的 f64 数组最小值查找
pub fn min_f64_simd(data: &[f64]) -> Option<f64> {
    if data.is_empty() {
        return None;
    }

    // 使用 4 路并行查找
    let mut min0 = f64::INFINITY;
    let mut min1 = f64::INFINITY;
    let mut min2 = f64::INFINITY;
    let mut min3 = f64::INFINITY;

    let chunks = data.chunks_exact(4);
    let remainder = chunks.remainder();

    for chunk in chunks {
        if chunk[0] < min0 {
            min0 = chunk[0];
        }
        if chunk[1] < min1 {
            min1 = chunk[1];
        }
        if chunk[2] < min2 {
            min2 = chunk[2];
        }
        if chunk[3] < min3 {
            min3 = chunk[3];
        }
    }

    let mut min_val = min0.min(min1).min(min2).min(min3);

    for &val in remainder {
        if val < min_val {
            min_val = val;
        }
    }

    Some(min_val)
}

/// 优化的 f64 数组最大值查找
pub fn max_f64_simd(data: &[f64]) -> Option<f64> {
    if data.is_empty() {
        return None;
    }

    // 使用 4 路并行查找
    let mut max0 = f64::NEG_INFINITY;
    let mut max1 = f64::NEG_INFINITY;
    let mut max2 = f64::NEG_INFINITY;
    let mut max3 = f64::NEG_INFINITY;

    let chunks = data.chunks_exact(4);
    let remainder = chunks.remainder();

    for chunk in chunks {
        if chunk[0] > max0 {
            max0 = chunk[0];
        }
        if chunk[1] > max1 {
            max1 = chunk[1];
        }
        if chunk[2] > max2 {
            max2 = chunk[2];
        }
        if chunk[3] > max3 {
            max3 = chunk[3];
        }
    }

    let mut max_val = max0.max(max1).max(max2).max(max3);

    for &val in remainder {
        if val > max_val {
            max_val = val;
        }
    }

    Some(max_val)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sum_f64_simd() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let sum = sum_f64_simd(&data);
        assert_eq!(sum, 15.0);
    }

    #[test]
    fn test_sum_f64_simd_empty() {
        let data: Vec<f64> = vec![];
        let sum = sum_f64_simd(&data);
        assert_eq!(sum, 0.0);
    }

    #[test]
    fn test_sum_f64_simd_large() {
        // 测试大于 4 的数据
        let data: Vec<f64> = (1..=100).map(|x| x as f64).collect();
        let sum = sum_f64_simd(&data);
        let expected: f64 = (1..=100).map(|x| x as f64).sum();
        assert!((sum - expected).abs() < 1e-10);
    }

    #[test]
    fn test_mean_f64_simd() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let mean = mean_f64_simd(&data).unwrap();
        assert_eq!(mean, 3.0);
    }

    #[test]
    fn test_mean_f64_simd_empty() {
        let data: Vec<f64> = vec![];
        assert!(mean_f64_simd(&data).is_none());
    }

    #[test]
    fn test_variance_f64_simd() {
        let data = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let variance = variance_f64_simd(&data).unwrap();
        // 样本方差 = 4.571428...
        assert!((variance - 4.571428571428571).abs() < 1e-10);
    }

    #[test]
    fn test_variance_f64_simd_small() {
        let data = vec![1.0];
        assert!(variance_f64_simd(&data).is_none());
    }

    #[test]
    fn test_std_dev_f64_simd() {
        let data = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let std_dev = std_dev_f64_simd(&data).unwrap();
        // 标准差 = sqrt(4.571428...) = 2.138...
        assert!((std_dev - 2.1380899352993947).abs() < 1e-10);
    }

    #[test]
    fn test_min_f64_simd() {
        let data = vec![5.0, 2.0, 8.0, 1.0, 9.0, 3.0];
        let min = min_f64_simd(&data).unwrap();
        assert_eq!(min, 1.0);
    }

    #[test]
    fn test_min_f64_simd_empty() {
        let data: Vec<f64> = vec![];
        assert!(min_f64_simd(&data).is_none());
    }

    #[test]
    fn test_max_f64_simd() {
        let data = vec![5.0, 2.0, 8.0, 1.0, 9.0, 3.0];
        let max = max_f64_simd(&data).unwrap();
        assert_eq!(max, 9.0);
    }

    #[test]
    fn test_max_f64_simd_empty() {
        let data: Vec<f64> = vec![];
        assert!(max_f64_simd(&data).is_none());
    }

    #[test]
    fn test_simd_consistency() {
        // 测试优化实现与标准实现的一致性
        let data: Vec<f64> = (1..=1000).map(|x| x as f64 * 0.1).collect();

        let simd_sum = sum_f64_simd(&data);
        let std_sum: f64 = data.iter().sum();
        assert!((simd_sum - std_sum).abs() < 1e-8);

        let simd_mean = mean_f64_simd(&data).unwrap();
        let std_mean = std_sum / data.len() as f64;
        assert!((simd_mean - std_mean).abs() < 1e-8);
    }

    #[test]
    fn test_unaligned_data() {
        // 测试非 4 的倍数的数据
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
        let sum = sum_f64_simd(&data);
        assert_eq!(sum, 28.0);

        let mean = mean_f64_simd(&data).unwrap();
        assert_eq!(mean, 4.0);
    }
}
