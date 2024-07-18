use std::future::IntoFuture;

use criterion::{criterion_group, criterion_main, Criterion};
use my_tiny_jpeg_decoder::get_jpeg_image;

fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("bench");
    group.bench_function("bench", |b| {
        b.iter(|| {
            async {
                _ = get_jpeg_image("s11138117.jpg".to_string())
                    .into_future()
                    .await;
            };
        })
    });
    group.finish();
}

criterion_group!(benches, bench,);
criterion_main!(benches);
