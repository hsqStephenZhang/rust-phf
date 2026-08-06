use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use phf::phf_map;
use std::time::Duration;

// ── 所有 map 使用统一 8 字节 key，只改变条目数 ──────────────────────────────
//
// tiny  ( 3 entries): disps.len == 1  ← 优化命中
// small ( 6 entries): disps.len == 2
// med   (12 entries): disps.len == 4
// large (30 entries): disps.len == 10

static TINY_MAP: phf::Map<&'static str, u32> = phf_map! {
    "key00001" => 1, "key00002" => 2, "key00003" => 3,
};

static SMALL_MAP: phf::Map<&'static str, u32> = phf_map! {
    "key00001" => 1, "key00002" => 2, "key00003" => 3,
    "key00004" => 4, "key00005" => 5, "key00006" => 6,
};

static MED_MAP: phf::Map<&'static str, u32> = phf_map! {
    "key00001" =>  1, "key00002" =>  2, "key00003" =>  3,
    "key00004" =>  4, "key00005" =>  5, "key00006" =>  6,
    "key00007" =>  7, "key00008" =>  8, "key00009" =>  9,
    "key00010" => 10, "key00011" => 11, "key00012" => 12,
};

static LARGE_MAP: phf::Map<&'static str, u32> = phf_map! {
    "key00001" =>  1, "key00002" =>  2, "key00003" =>  3,
    "key00004" =>  4, "key00005" =>  5, "key00006" =>  6,
    "key00007" =>  7, "key00008" =>  8, "key00009" =>  9,
    "key00010" => 10, "key00011" => 11, "key00012" => 12,
    "key00013" => 13, "key00014" => 14, "key00015" => 15,
    "key00016" => 16, "key00017" => 17, "key00018" => 18,
    "key00019" => 19, "key00020" => 20, "key00021" => 21,
    "key00022" => 22, "key00023" => 23, "key00024" => 24,
    "key00025" => 25, "key00026" => 26, "key00027" => 27,
    "key00028" => 28, "key00029" => 29, "key00030" => 30,
};

// ── USIZE KEY MAPS ─────────────────────────────────────────────────────────

static TINY_MAP_USIZE: phf::Map<usize, u32> = phf_map! {
    1usize => 1, 2usize => 2, 3usize => 3,
};

static SMALL_MAP_USIZE: phf::Map<usize, u32> = phf_map! {
    1usize => 1, 2usize => 2, 3usize => 3,
    4usize => 4, 5usize => 5, 6usize => 6,
};

static MED_MAP_USIZE: phf::Map<usize, u32> = phf_map! {
    1usize =>  1, 2usize =>  2, 3usize =>  3,
    4usize =>  4, 5usize =>  5, 6usize =>  6,
    7usize =>  7, 8usize =>  8, 9usize =>  9,
    10usize => 10, 11usize => 11, 12usize => 12,
};

static LARGE_MAP_USIZE: phf::Map<usize, u32> = phf_map! {
    1usize =>  1, 2usize =>  2, 3usize =>  3,
    4usize =>  4, 5usize =>  5, 6usize =>  6,
    7usize =>  7, 8usize =>  8, 9usize =>  9,
    10usize => 10, 11usize => 11, 12usize => 12,
    13usize => 13, 14usize => 14, 15usize => 15,
    16usize => 16, 17usize => 17, 18usize => 18,
    19usize => 19, 20usize => 20, 21usize => 21,
    22usize => 22, 23usize => 23, 24usize => 24,
    25usize => 25, 26usize => 26, 27usize => 27,
    28usize => 28, 29usize => 29, 30usize => 30,
};

// ── 批量查询集合（各 32 条，循环覆盖所有 key，消除单 key 缓存偏差）─────────
const HITS: &[&str] = &[
    "key00001", "key00002", "key00003", "key00001", "key00002", "key00003", "key00001", "key00002",
    "key00003", "key00001", "key00002", "key00003", "key00001", "key00002", "key00003", "key00001",
    "key00002", "key00003", "key00001", "key00002", "key00003", "key00001", "key00002", "key00003",
    "key00001", "key00002", "key00003", "key00001", "key00002", "key00003", "key00001", "key00002",
];

const MISSES: &[&str] = &[
    "mis00001", "mis00002", "mis00003", "mis00004", "mis00005", "mis00006", "mis00007", "mis00008",
    "mis00009", "mis00010", "mis00011", "mis00012", "mis00013", "mis00014", "mis00015", "mis00016",
    "mis00017", "mis00018", "mis00019", "mis00020", "mis00021", "mis00022", "mis00023", "mis00024",
    "mis00025", "mis00026", "mis00027", "mis00028", "mis00029", "mis00030", "mis00031", "mis00032",
];

const HITS_USIZE: &[usize] = &[
    1, 2, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3, 1, 2,
];

const MISSES_USIZE: &[usize] = &[
    100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116, 117, 118,
    119, 120, 121, 122, 123, 124, 125, 126, 127, 128, 129, 130, 131,
];

// ── 通用 bench helpers ───────────────────────────────────────────────────────

fn bench_map<F>(c: &mut Criterion, group_name: &str, disps_len: usize, mut lookup: F)
where
    F: FnMut(&str) -> Option<u32>,
{
    let label = format!("{group_name} (disps.len={disps_len})");
    let mut g = c.benchmark_group(&label);
    g.warm_up_time(Duration::from_secs(5));
    g.measurement_time(Duration::from_secs(10));
    g.sample_size(500);

    g.throughput(Throughput::Elements(HITS.len() as u64));
    g.bench_function("hit_batch", |b| {
        b.iter(|| {
            let mut sum = 0u32;
            for key in HITS {
                sum = sum.wrapping_add(lookup(black_box(key)).unwrap_or(0));
            }
            black_box(sum)
        })
    });

    g.throughput(Throughput::Elements(MISSES.len() as u64));
    g.bench_function("miss_batch", |b| {
        b.iter(|| {
            let mut found = 0u32;
            for key in MISSES {
                found = found.wrapping_add(lookup(black_box(key)).is_some() as u32);
            }
            black_box(found)
        })
    });

    g.finish();
}

fn bench_map_usize<F>(c: &mut Criterion, group_name: &str, disps_len: usize, mut lookup: F)
where
    F: FnMut(&usize) -> Option<u32>,
{
    let label = format!("{group_name}_usize (disps.len={disps_len})");
    let mut g = c.benchmark_group(&label);
    g.warm_up_time(Duration::from_secs(5));
    g.measurement_time(Duration::from_secs(10));
    g.sample_size(500);

    g.throughput(Throughput::Elements(HITS_USIZE.len() as u64));
    g.bench_function("hit_batch", |b| {
        b.iter(|| {
            let mut sum = 0u32;
            for key in HITS_USIZE {
                sum = sum.wrapping_add(lookup(black_box(key)).unwrap_or(0));
            }
            black_box(sum)
        })
    });

    g.throughput(Throughput::Elements(MISSES_USIZE.len() as u64));
    g.bench_function("miss_batch", |b| {
        b.iter(|| {
            let mut found = 0u32;
            for key in MISSES_USIZE {
                found = found.wrapping_add(lookup(black_box(key)).is_some() as u32);
            }
            black_box(found)
        })
    });

    g.finish();
}

// ── Benches ──────────────────────────────────────────────────────────────────

fn bench_str_keys(c: &mut Criterion) {
    bench_map(c, "tiny", 1, |k| TINY_MAP.get(k).copied());
    bench_map(c, "small", 2, |k| SMALL_MAP.get(k).copied());
    bench_map(c, "med", 4, |k| MED_MAP.get(k).copied());
    bench_map(c, "large", 10, |k| LARGE_MAP.get(k).copied());
}

fn bench_usize_keys(c: &mut Criterion) {
    bench_map_usize(c, "tiny", 1, |k| TINY_MAP_USIZE.get(k).copied());
    bench_map_usize(c, "small", 2, |k| SMALL_MAP_USIZE.get(k).copied());
    bench_map_usize(c, "med", 4, |k| MED_MAP_USIZE.get(k).copied());
    bench_map_usize(c, "large", 10, |k| LARGE_MAP_USIZE.get(k).copied());
}

criterion_group!(benches, bench_str_keys, bench_usize_keys);
criterion_main!(benches);
