use fimg::*;

macro_rules! bench {
    (fn $name: ident() { run $fn: ident() } $($namec:ident)?) => {
        fn $name() {
            let mut img: Image<_, 4> =
                Image::build(128, 128).buf(include_bytes!("4_128x128.imgbuf").to_vec());
            for _ in 0..256 {
                #[allow(unused_unsafe)]
                unsafe {
                    img.$fn()
                };
            }
        }

        $(fn $namec() {
            let img: Image<&[u8], 4> =
                Image::build(128, 128).buf(include_bytes!("4_128x128.imgbuf"));
            #[allow(unused_unsafe)]
            unsafe {
                std::hint::black_box(img.cloner().$fn())
            };
        })?
    };
}

bench!(fn flip_h() { run flip_h() } flip_hc);
bench!(fn flip_v() { run flip_v() } flip_vc);
bench!(fn rotate_90() { run rot_90() } rot_90c);
bench!(fn rotate_180() { run rot_180() } rot_180c);
bench!(fn rotate_270() { run rot_270() } rot_270c);
iai::main!(
    flip_h, flip_v, rotate_90, rotate_180, rotate_270, flip_hc, flip_vc, rot_90c, rot_180c,
    rot_270c
);
