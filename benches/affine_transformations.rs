use fimg::*;

macro_rules! bench {
    (fn $name: ident() { run $fn: ident() }) => {
        fn $name() {
            let mut img: Image<_, 4> =
                Image::build(160, 160).buf(include_bytes!("4_160x160.imgbuf").to_vec());
            for _ in 0..256 {
                #[allow(unused_unsafe)]
                unsafe {
                    img.$fn()
                };
            }
        }
    };
}

bench!(fn flip_h() { run flip_h() });
bench!(fn flip_v() { run flip_v() });
bench!(fn rotate_90() { run rot_90() });
bench!(fn rotate_180() { run rot_180() });
bench!(fn rotate_270() { run rot_270() });
iai::main!(flip_h, flip_v, rotate_90, rotate_180, rotate_270);
