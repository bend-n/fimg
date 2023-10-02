use fimg::*;
fn repeat() {
    let x: Image<&[u8], 3> = Image::build(8, 8).buf(include_bytes!("3_8x8.imgbuf"));
    let _ = unsafe { x.repeated(128, 128) }; // repeat 16 times
}
iai::main!(repeat);
