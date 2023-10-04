use fimg::*;
fn tri() {
    let mut b = [0u8; 1000 * 1000 * 4];
    let mut i = Image::<&mut [u8], 4>::build(1000, 1000).buf(&mut b);
    i.tri((0., 0.), (1000., 500.), (0., 999.), [255, 255, 255, 255]);
}
iai::main!(tri);
