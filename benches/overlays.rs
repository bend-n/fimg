use criterion::{criterion_group, criterion_main, Criterion};
use fimg::*;

pub fn criterion_benchmark(bench: &mut Criterion) {
    let mut group = bench.benchmark_group("overlays");
    {
        let mut a: Image<_, 3> = Image::alloc(64, 64);
        let b = Image::<&[u8], 3>::new(
            4.try_into().unwrap(),
            4.try_into().unwrap(),
            *&include_bytes!("3_4x4.imgbuf"),
        );
        group.bench_function("overlay 3x3 offset", |bench| {
            bench.iter(|| unsafe {
                for x in 0..16 {
                    for y in 0..16 {
                        a.as_mut().overlay_at(&b, x * 4, y * 4);
                    }
                }
            });
        });
        assert_eq!(a.as_ref().buffer, include_bytes!("3x3_at_out.imgbuf"));
    }
    {
        let mut a: Image<_, 3> = Image::alloc(64, 64);
        let b = Image::<&[u8], 4>::new(
            4.try_into().unwrap(),
            4.try_into().unwrap(),
            *&include_bytes!("4_4x4.imgbuf"),
        );
        group.bench_function("overlay 4x3 offset", |bench| {
            bench.iter(|| unsafe {
                for x in 0..16 {
                    for y in 0..16 {
                        a.as_mut().overlay_at(&b, x * 4, y * 4);
                    }
                }
            });
        });

        assert_eq!(a.as_ref().buffer, include_bytes!("4x3_at_out.imgbuf"));
    }
    {
        let mut a: Image<_, 4> = Image::alloc(64, 64);
        let b = Image::<&[u8], 4>::new(
            4.try_into().unwrap(),
            4.try_into().unwrap(),
            *&include_bytes!("4_4x4.imgbuf"),
        );
        group.bench_function("overlay 4x4 offset", |bench| {
            bench.iter(|| unsafe {
                for x in 0..16 {
                    for y in 0..16 {
                        a.as_mut().overlay_at(&b, x * 4, y * 4);
                    }
                }
            });
        });
        assert_eq!(a.as_ref().buffer, include_bytes!("4x4_at_out.imgbuf"));
    }
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
