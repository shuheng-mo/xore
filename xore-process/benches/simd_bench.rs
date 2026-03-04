use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use xore_process::simd::*;

fn bench_sum(c: &mut Criterion) {
    let mut group = c.benchmark_group("sum");

    for size in [100, 1_000, 10_000, 100_000, 1_000_000].iter() {
        let data: Vec<f64> = (0..*size).map(|x| x as f64).collect();

        group.throughput(Throughput::Elements(*size as u64));

        // SIMD 版本
        group.bench_with_input(BenchmarkId::new("simd", size), &data, |b, data| {
            b.iter(|| sum_f64_simd(black_box(data)))
        });

        // 标准版本（对比）
        group.bench_with_input(BenchmarkId::new("std", size), &data, |b, data| {
            b.iter(|| data.iter().sum::<f64>())
        });
    }

    group.finish();
}

fn bench_mean(c: &mut Criterion) {
    let mut group = c.benchmark_group("mean");

    for size in [100, 1_000, 10_000, 100_000, 1_000_000].iter() {
        let data: Vec<f64> = (0..*size).map(|x| x as f64).collect();

        group.throughput(Throughput::Elements(*size as u64));

        // SIMD 版本
        group.bench_with_input(BenchmarkId::new("simd", size), &data, |b, data| {
            b.iter(|| mean_f64_simd(black_box(data)))
        });

        // 标准版本
        group.bench_with_input(BenchmarkId::new("std", size), &data, |b, data| {
            b.iter(|| {
                let sum: f64 = data.iter().sum();
                sum / data.len() as f64
            })
        });
    }

    group.finish();
}

fn bench_variance(c: &mut Criterion) {
    let mut group = c.benchmark_group("variance");

    for size in [100, 1_000, 10_000, 100_000].iter() {
        let data: Vec<f64> = (0..*size).map(|x| x as f64 * 0.1).collect();

        group.throughput(Throughput::Elements(*size as u64));

        // SIMD 版本
        group.bench_with_input(BenchmarkId::new("simd", size), &data, |b, data| {
            b.iter(|| variance_f64_simd(black_box(data)))
        });

        // 标准版本
        group.bench_with_input(BenchmarkId::new("std", size), &data, |b, data| {
            b.iter(|| {
                let mean: f64 = data.iter().sum::<f64>() / data.len() as f64;
                let variance: f64 = data
                    .iter()
                    .map(|&x| {
                        let diff = x - mean;
                        diff * diff
                    })
                    .sum::<f64>()
                    / (data.len() - 1) as f64;
                variance
            })
        });
    }

    group.finish();
}

fn bench_min_max(c: &mut Criterion) {
    let mut group = c.benchmark_group("min_max");

    for size in [100, 1_000, 10_000, 100_000, 1_000_000].iter() {
        let data: Vec<f64> = (0..*size).map(|x| (x as f64 * 0.7) % 1000.0).collect();

        group.throughput(Throughput::Elements(*size as u64));

        // SIMD min
        group.bench_with_input(BenchmarkId::new("min_simd", size), &data, |b, data| {
            b.iter(|| min_f64_simd(black_box(data)))
        });

        // 标准 min
        group.bench_with_input(BenchmarkId::new("min_std", size), &data, |b, data| {
            b.iter(|| data.iter().copied().min_by(|a, b| a.partial_cmp(b).unwrap()))
        });

        // SIMD max
        group.bench_with_input(BenchmarkId::new("max_simd", size), &data, |b, data| {
            b.iter(|| max_f64_simd(black_box(data)))
        });

        // 标准 max
        group.bench_with_input(BenchmarkId::new("max_std", size), &data, |b, data| {
            b.iter(|| data.iter().copied().max_by(|a, b| a.partial_cmp(b).unwrap()))
        });
    }

    group.finish();
}

criterion_group!(benches, bench_sum, bench_mean, bench_variance, bench_min_max);
criterion_main!(benches);
