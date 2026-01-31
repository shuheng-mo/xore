//! 分配器性能基准测试
//!
//! 测试 mimalloc vs 系统分配器在 XORE 典型工作负载下的性能差异。
//!
//! 运行方式:
//! - 使用 mimalloc: `cargo bench --features mimalloc`
//! - 使用系统分配器: `cargo bench`

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::collections::HashMap;
use xore_search::{FileScanner, FileTypeFilter, ScanConfig};

/// 测试 Vec<String> 分配性能
fn bench_vec_allocations(c: &mut Criterion) {
    let mut group = c.benchmark_group("vec_allocations");

    for size in [1_000, 10_000, 100_000, 1_000_000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let mut v: Vec<String> = Vec::with_capacity(size);
                for i in 0..size {
                    v.push(format!("path/to/file_{}.txt", i));
                }
                black_box(v)
            });
        });
    }
    group.finish();
}

/// 测试 HashMap 操作性能（典型的索引构建场景）
fn bench_hashmap_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("hashmap_operations");

    for size in [1_000, 10_000, 100_000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let mut map: HashMap<String, usize> = HashMap::with_capacity(size);
                for i in 0..size {
                    map.insert(format!("key_{}", i), i);
                }
                // 模拟查找操作
                for i in 0..size {
                    black_box(map.get(&format!("key_{}", i)));
                }
                black_box(map)
            });
        });
    }
    group.finish();
}

/// 测试不同大小字符串的分配模式
fn bench_string_allocations(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_allocations");

    // 小字符串（文件名）
    group.bench_function("small_strings_10k", |b| {
        b.iter(|| {
            let mut strings: Vec<String> = Vec::with_capacity(10_000);
            for i in 0..10_000 {
                strings.push(format!("file_{}.rs", i));
            }
            black_box(strings)
        });
    });

    // 中等字符串（文件路径）
    group.bench_function("medium_strings_10k", |b| {
        b.iter(|| {
            let mut strings: Vec<String> = Vec::with_capacity(10_000);
            for i in 0..10_000 {
                strings.push(format!("/Users/test/project/src/module_{}/file_{}.rs", i % 100, i));
            }
            black_box(strings)
        });
    });

    // 大字符串（文件内容片段）
    group.bench_function("large_strings_1k", |b| {
        let content = "x".repeat(1024); // 1KB 字符串
        b.iter(|| {
            let mut strings: Vec<String> = Vec::with_capacity(1000);
            for _ in 0..1000 {
                strings.push(content.clone());
            }
            black_box(strings)
        });
    });

    group.finish();
}

/// 测试真实文件扫描场景
fn bench_file_scanning(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_scanning");

    // 扫描当前目录
    group.bench_function("scan_cwd", |b| {
        b.iter(|| {
            let config = ScanConfig::new(".").with_max_depth(5).with_respect_gitignore(true);
            let scanner = FileScanner::new(config);
            let result = scanner.scan();
            black_box(result)
        });
    });

    // 扫描代码文件
    group.bench_function("scan_code_files", |b| {
        b.iter(|| {
            let config = ScanConfig::new(".")
                .with_max_depth(5)
                .with_file_type(FileTypeFilter::Code)
                .with_respect_gitignore(true);
            let scanner = FileScanner::new(config);
            let result = scanner.scan();
            black_box(result)
        });
    });

    group.finish();
}

/// 测试混合工作负载（模拟搜索索引构建）
fn bench_mixed_workload(c: &mut Criterion) {
    let mut group = c.benchmark_group("mixed_workload");

    group.bench_function("search_index_simulation", |b| {
        b.iter(|| {
            // 模拟构建搜索索引
            let mut file_paths: Vec<String> = Vec::with_capacity(5000);
            let mut content_index: HashMap<String, Vec<usize>> = HashMap::new();

            // 添加文件路径
            for i in 0..5000 {
                file_paths.push(format!("/project/src/module_{}/file_{}.rs", i % 50, i));
            }

            // 构建倒排索引
            let keywords = ["fn", "struct", "impl", "use", "mod", "pub", "let", "mut"];
            for (idx, _path) in file_paths.iter().enumerate() {
                for keyword in keywords.iter() {
                    content_index.entry(keyword.to_string()).or_insert_with(Vec::new).push(idx);
                }
            }

            // 模拟查询
            for keyword in keywords.iter() {
                black_box(content_index.get(*keyword));
            }

            black_box((file_paths, content_index))
        });
    });

    group.finish();
}

/// 测试频繁的小分配和释放
fn bench_allocation_churn(c: &mut Criterion) {
    let mut group = c.benchmark_group("allocation_churn");

    group.bench_function("small_alloc_free_cycle", |b| {
        b.iter(|| {
            for _ in 0..10_000 {
                let s = String::from("temporary allocation");
                black_box(&s);
                // s 在这里被释放
            }
        });
    });

    group.bench_function("vec_grow_shrink", |b| {
        b.iter(|| {
            let mut v: Vec<i32> = Vec::new();
            for i in 0..10_000 {
                v.push(i);
            }
            while !v.is_empty() {
                v.pop();
            }
            black_box(v)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_vec_allocations,
    bench_hashmap_operations,
    bench_string_allocations,
    bench_file_scanning,
    bench_mixed_workload,
    bench_allocation_churn,
);

criterion_main!(benches);
