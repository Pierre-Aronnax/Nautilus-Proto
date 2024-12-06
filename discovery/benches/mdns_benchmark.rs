use criterion::{criterion_group, criterion_main, Criterion,black_box};

fn bench_service_announcement(c: &mut Criterion) {
    c.bench_function("service_announcement", |b| {
        b.iter(|| {
            // Black box the iteration to ensure it doesn't get optimized away.
            black_box("Example service test");
        });
    });
}

criterion_group!(benches, bench_service_announcement);
criterion_main!(benches);
