#![feature(test)]
extern crate test;
use fimg::*;
use test::Bencher;

#[bench]
fn overlay_3on3at(bench: &mut Bencher) {
    let mut v = vec![0u8; 3 * 64 * 64];
    let mut a: Image<_, 3> = Image::new(
        64.try_into().unwrap(),
        64.try_into().unwrap(),
        v.as_mut_slice(),
    );
    let b = Image::<&[u8], 3>::new(
        4.try_into().unwrap(),
        4.try_into().unwrap(),
        *&include_bytes!("3_4x4.imgbuf"),
    );
    bench.iter(|| unsafe {
        for x in 0..16 {
            for y in 0..16 {
                a.overlay_at(&b, x * 4, y * 4);
            }
        }
    });
    assert_eq!(a.as_ref().buffer, include_bytes!("3x3_at_out.imgbuf"));
}

#[bench]
fn overlay_4on3at(bench: &mut Bencher) {
    let mut a: Image<_, 3> = Image::alloc(64, 64);
    let b = Image::<&[u8], 4>::new(
        4.try_into().unwrap(),
        4.try_into().unwrap(),
        *&include_bytes!("4_4x4.imgbuf"),
    );
    bench.iter(|| unsafe {
        for x in 0..16 {
            for y in 0..16 {
                a.as_mut().overlay_at(&b, x * 4, y * 4);
            }
        }
    });
    assert_eq!(a.as_ref().buffer, include_bytes!("4x3_at_out.imgbuf"));
}

#[bench]
fn overlay_4on4at(bench: &mut Bencher) {
    let mut a: Image<_, 4> = Image::alloc(64, 64);
    let b = Image::<&[u8], 4>::new(
        4.try_into().unwrap(),
        4.try_into().unwrap(),
        *&include_bytes!("4_4x4.imgbuf"),
    );
    bench.iter(|| unsafe {
        for x in 0..16 {
            for y in 0..16 {
                a.as_mut().overlay_at(&b, x * 4, y * 4);
            }
        }
    });
    assert_eq!(a.as_ref().buffer, include_bytes!("4x4_at_out.imgbuf"));
}
