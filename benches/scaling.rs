use fimg::{scale::*, Image};

macro_rules! bench {
    ($([$a: ident, $alg:ident]),+ $(,)?) => {
        $(fn $a() {
            let img: Image<_, 3> = Image::open("tdata/cat.png");
            iai::black_box(img.scale::<$alg>(267, 178));
        })+

        iai::main!($($a,)+);
    };
}
bench![
    [nearest, Nearest],
    [bilinear, Bilinear],
    [boxs, Box],
    [lanczos3, Lanczos3],
    [catmull, CatmullRom],
    [mitchell, Mitchell],
    [hamming, Hamming],
];
