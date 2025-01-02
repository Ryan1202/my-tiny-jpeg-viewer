use std::future::IntoFuture;

use criterion::{criterion_group, criterion_main, Criterion};
use my_tiny_jpeg_decoder::get_jpeg_image;

fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("jpeg_decode_bench");
    group.bench_function("jpeg_decode", |b| {
        b.iter(|| {
            get_jpeg_image("Rheinfall.jpg".to_string());
        })
    });
    group.finish();
}

fn bench_idct(c: &mut Criterion) {
    let mut group = c.benchmark_group("idct_bench");
    let dct = my_tiny_jpeg_decoder::decode::dct::DCT::new();
    let mut data = [[1f32; 8]; 8];
    
    group.bench_function("idct", |b| {
        b.iter(|| {
            dct.idct2d(data);
        })
    });
    group.finish();
}

criterion_group!(benches, bench, bench_idct);
criterion_main!(benches);
