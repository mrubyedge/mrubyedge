use criterion::{Criterion, black_box, criterion_group, criterion_main};
use fnv::FnvHashMap;
use once_cell::sync::Lazy;
use std::collections::HashMap;

// 事前に生成された静的な文字列を使用
macro_rules! generate_static_keys {
    ($name:ident, $len:expr, $count:expr) => {
        static $name: Lazy<Vec<String>> = Lazy::new(|| {
            (0..$count)
                .map(|i| {
                    let base = format!("k{}", i);
                    if base.len() >= $len {
                        base.chars().take($len).collect()
                    } else {
                        format!("{:0<width$}", base, width = $len)
                    }
                })
                .collect()
        });
    };
}

generate_static_keys!(KEYS_1CHAR, 1, 10000);
generate_static_keys!(KEYS_5CHAR, 5, 10000);
generate_static_keys!(KEYS_10CHAR, 10, 10000);
generate_static_keys!(KEYS_20CHAR, 20, 10000);
generate_static_keys!(KEYS_50CHAR, 50, 10000);

static VALUES: Lazy<Vec<String>> = Lazy::new(|| (0..10000).map(|i| format!("v{}", i)).collect());

// 1 char keys
fn bench_default_1char(c: &mut Criterion) {
    c.bench_function("default HashMap 10000 entries (1 char keys)", |b| {
        b.iter(|| {
            let mut map = HashMap::new();
            for i in 0..10000 {
                map.insert(
                    black_box(KEYS_1CHAR[i].clone()),
                    black_box(VALUES[i].clone()),
                );
            }
            map
        })
    });
}

fn bench_fnv_1char(c: &mut Criterion) {
    c.bench_function("FNV HashMap 10000 entries (1 char keys)", |b| {
        b.iter(|| {
            let mut map = FnvHashMap::default();
            for i in 0..10000 {
                map.insert(
                    black_box(KEYS_1CHAR[i].clone()),
                    black_box(VALUES[i].clone()),
                );
            }
            map
        })
    });
}

// 5 char keys
fn bench_default_5char(c: &mut Criterion) {
    c.bench_function("default HashMap 10000 entries (5 char keys)", |b| {
        b.iter(|| {
            let mut map = HashMap::new();
            for i in 0..10000 {
                map.insert(
                    black_box(KEYS_5CHAR[i].clone()),
                    black_box(VALUES[i].clone()),
                );
            }
            map
        })
    });
}

fn bench_fnv_5char(c: &mut Criterion) {
    c.bench_function("FNV HashMap 10000 entries (5 char keys)", |b| {
        b.iter(|| {
            let mut map = FnvHashMap::default();
            for i in 0..10000 {
                map.insert(
                    black_box(KEYS_5CHAR[i].clone()),
                    black_box(VALUES[i].clone()),
                );
            }
            map
        })
    });
}

// 10 char keys
fn bench_default_10char(c: &mut Criterion) {
    c.bench_function("default HashMap 10000 entries (10 char keys)", |b| {
        b.iter(|| {
            let mut map = HashMap::new();
            for i in 0..10000 {
                map.insert(
                    black_box(KEYS_10CHAR[i].clone()),
                    black_box(VALUES[i].clone()),
                );
            }
            map
        })
    });
}

fn bench_fnv_10char(c: &mut Criterion) {
    c.bench_function("FNV HashMap 10000 entries (10 char keys)", |b| {
        b.iter(|| {
            let mut map = FnvHashMap::default();
            for i in 0..10000 {
                map.insert(
                    black_box(KEYS_10CHAR[i].clone()),
                    black_box(VALUES[i].clone()),
                );
            }
            map
        })
    });
}

// 20 char keys
fn bench_default_20char(c: &mut Criterion) {
    c.bench_function("default HashMap 10000 entries (20 char keys)", |b| {
        b.iter(|| {
            let mut map = HashMap::new();
            for i in 0..10000 {
                map.insert(
                    black_box(KEYS_20CHAR[i].clone()),
                    black_box(VALUES[i].clone()),
                );
            }
            map
        })
    });
}

fn bench_fnv_20char(c: &mut Criterion) {
    c.bench_function("FNV HashMap 10000 entries (20 char keys)", |b| {
        b.iter(|| {
            let mut map = FnvHashMap::default();
            for i in 0..10000 {
                map.insert(
                    black_box(KEYS_20CHAR[i].clone()),
                    black_box(VALUES[i].clone()),
                );
            }
            map
        })
    });
}

// 50 char keys
fn bench_default_50char(c: &mut Criterion) {
    c.bench_function("default HashMap 10000 entries (50 char keys)", |b| {
        b.iter(|| {
            let mut map = HashMap::new();
            for i in 0..10000 {
                map.insert(
                    black_box(KEYS_50CHAR[i].clone()),
                    black_box(VALUES[i].clone()),
                );
            }
            map
        })
    });
}

fn bench_fnv_50char(c: &mut Criterion) {
    c.bench_function("FNV HashMap 10000 entries (50 char keys)", |b| {
        b.iter(|| {
            let mut map = FnvHashMap::default();
            for i in 0..10000 {
                map.insert(
                    black_box(KEYS_50CHAR[i].clone()),
                    black_box(VALUES[i].clone()),
                );
            }
            map
        })
    });
}

criterion_group!(
    benches,
    bench_fnv_1char,
    bench_default_1char,
    bench_fnv_5char,
    bench_default_5char,
    bench_fnv_10char,
    bench_default_10char,
    bench_fnv_20char,
    bench_default_20char,
    bench_fnv_50char,
    bench_default_50char,
);
criterion_main!(benches);
