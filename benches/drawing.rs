use fimg::*;
fn tri() {
    let mut i: Image<_, 4> = fimg::make!(4 channels 1000 x 1000).boxed();
    i.tri((0., 0.), (1000., 500.), (0., 999.), [255, 255, 255, 255]);
    iai::black_box(i);
}
fn line() {
    let mut i: Image<_, 4> = fimg::make!(4 channels 500 x 750).boxed();
    i.line((-50, 20), (550, 800), [255, 165, 0, 255]);
    i.save("z.png");
    iai::black_box(i);
}
iai::main!(tri, line);
