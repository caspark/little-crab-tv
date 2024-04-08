use criterion::{black_box, criterion_group, criterion_main, Criterion};

use crab_tv::{Canvas, WHITE};
use glam::IVec2;
use rgb::RGBA8;

fn line_drawing(c: &mut Criterion) {
    let mut group = c.benchmark_group("line-drawing");

    group.bench_function("v1-slow", |b| {
        let mut image = Canvas::new(100, 100);
        b.iter(|| {
            image.line_slow(0, 0, 99, 99, RGBA8::new(255, 0, 0));
        });
        black_box(image);
    });

    group.bench_function("v2-faster", |b| {
        let mut image = Canvas::new(100, 100);
        b.iter(|| {
            image.line_faster(0, 0, 99, 99, RGBA8::new(255, 0, 0));
        });
        black_box(image);
    });

    group.bench_function("v3-integer maths", |b| {
        let mut image = Canvas::new(100, 100);
        b.iter(|| {
            image.line_fastest(0, 0, 99, 99, RGBA8::new(255, 0, 0));
        });
        black_box(image);
    });

    group.finish();
}

fn triangle_drawing(c: &mut Criterion) {
    let mut group = c.benchmark_group("triangle-drawing");

    let t = [IVec2::new(0, 10), IVec2::new(7, 50), IVec2::new(100, 30)];

    group.bench_function("v1-sweep-verbose", |b| {
        let mut image = Canvas::new(100, 100);
        b.iter(|| {
            image.triangle_linesweep_verbose(&t, WHITE);
        });
        black_box(image);
    });

    group.bench_function("v2-sweep-compact", |b| {
        let mut image = Canvas::new(100, 100);
        b.iter(|| {
            image.triangle_linesweep_compact(&t, WHITE);
        });
        black_box(image);
    });

    group.bench_function("v3-barycentric", |b| {
        let mut image = Canvas::new(100, 100);
        b.iter(|| {
            image.triangle_barycentric(&t, WHITE);
        });
        black_box(image);
    });

    group.finish();
}

criterion_group!(benches, line_drawing, triangle_drawing);
criterion_main!(benches);
