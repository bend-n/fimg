use fimg::*;
fn tri() {
    let mut i: Image<_, 4> = fimg::make!(4 channels 1000 x 1000);
    i.as_mut()
        .tri((0., 0.), (1000., 500.), (0., 999.), [255, 255, 255, 255]);
}
iai::main!(tri);
