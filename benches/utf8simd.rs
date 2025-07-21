use criterion::{Criterion, criterion_group, criterion_main};

use core::hint::black_box;

fn bench(c: &mut Criterion) {
    // 1 GB
    let size = 1_000_000_000;

    // create a large UTF-8 byte array with mixed content
    let text = "Hello, 世界! 🌍 This is a UTF-8 benchmark with emoji 🚀 and Unicode characters: αβγδε ñ\n";

    let mut data = Vec::with_capacity(size);
    for _ in 0..(size / text.len()) {
        data.extend_from_slice(text.as_bytes());
    }

    // slices with different alignment
    let slices = [
        &data[..],
        &data[1..],
        &data[2..],
        &data[3..],
        &data[4..],
        &data[5..],
    ];

    // benchmark
    let mut group = c.benchmark_group("validation");
    group.throughput(criterion::Throughput::BytesDecimal((data.len() * slices.len()) as u64));
    group.sample_size(10);

    // utf8simd
    group.bench_function("utf8simd", |b| {
        b.iter(|| {
            for (idx, &slice) in slices.iter().enumerate() {
                let str = utf8simd::from_utf8(black_box(slice)).unwrap();
                assert_eq!(str.len(), data.len() - idx);
            }
        })
    });

    // core
    group.bench_function("core", |b| {
        b.iter(|| {
            for (idx, &slice) in slices.iter().enumerate() {
                let str = core::str::from_utf8(black_box(slice)).unwrap();
                assert_eq!(str.len(), data.len() - idx);
            }
        })
    });

    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
