use criterion::{black_box, criterion_group, criterion_main, Criterion};

use crab_tv::Canvas;
use rgb::RGB8;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("line slow", |b| {
        let mut image = Canvas::new(100, 100);
        b.iter(|| {
            image.line_slow(0, 0, 99, 99, RGB8::new(255, 0, 0));
        });
        black_box(image);
    });

    c.bench_function("line faster", |b| {
        let mut image = Canvas::new(100, 100);
        b.iter(|| {
            image.line_faster(0, 0, 99, 99, RGB8::new(255, 0, 0));
        });
        black_box(image);
    });

    c.bench_function("line integer maths", |b| {
        let mut image = Canvas::new(100, 100);
        b.iter(|| {
            image.line_fastest(0, 0, 99, 99, RGB8::new(255, 0, 0));
        });
        black_box(image);
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
