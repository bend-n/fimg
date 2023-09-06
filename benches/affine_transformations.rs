#![feature(test)]
extern crate test;
use fimg::*;
use test::Bencher;

macro_rules! bench {
    (fn $name: ident() { run $fn: ident() }) => {
        #[bench]
        fn $name(b: &mut Bencher) {
            let mut img: Image<_, 4> =
                Image::build(64, 64).buf(include_bytes!("4_180x180.imgbuf").to_vec());
            #[allow(unused_unsafe)]
            b.iter(|| unsafe {
                for _ in 0..256 {
                    img.$fn();
                }
            });
        }
    };
}

bench!(fn flip_h() { run flip_h() });
bench!(fn flip_v() { run flip_v() });
bench!(fn rotate_90() { run rot_90() });
bench!(fn rotate_180() { run rot_180() });
bench!(fn rotate_270() { run rot_270() });
