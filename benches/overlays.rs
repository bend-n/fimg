use fimg::*;

fn overlay_3on3at() {
    let mut a: Image<_, 3> = Image::alloc(128, 128);
    let b: Image<&[u8], 3> = Image::build(8, 8).buf(include_bytes!("3_8x8.imgbuf"));
    for x in 0..16 {
        for y in 0..16 {
            unsafe { a.as_mut().overlay_at(&b, x * 8, y * 8) };
        }
    }
}

fn overlay_4on3at() {
    let mut a: Image<_, 3> = Image::alloc(128, 128);
    let b: Image<&[u8], 4> = Image::build(8, 8).buf(include_bytes!("4_8x8.imgbuf"));
    for x in 0..16 {
        for y in 0..16 {
            unsafe { a.as_mut().overlay_at(&b, x * 8, y * 8) };
        }
    }
}

fn overlay_4on4at() {
    let mut a: Image<_, 4> = Image::alloc(128, 128);
    let b: Image<&[u8], 4> = Image::build(8, 8).buf(include_bytes!("4_8x8.imgbuf"));
    for x in 0..16 {
        for y in 0..16 {
            unsafe { a.as_mut().overlay_at(&b, x * 8, y * 8) };
        }
    }
}
iai::main!(overlay_3on3at, overlay_4on3at, overlay_4on4at);
