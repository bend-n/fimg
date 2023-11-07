//! holds scaling operations.
//!
//! choose from the wide expanse of options (ordered fastest to slowest):
//!
//! - [`Nearest`]: quickest, dumbest, jaggedest, scaling algorithm
//! - [`Box`]: you want slightly less pixels than nearest? here you go! kinda blurry though.
//! - [`Bilinear`]: _smooth_ scaling algorithm. rather fuzzy.
//! - [`Hamming`]: solves the [`Box`] problems. clearer image.
//! - [`CatmullRom`]: about the same as [`Hamming`], just a little slower.
//! - [`Mitchell`]: honestly, cant see the difference from [`CatmullRom`].
//! - [`Lanczos3`]: prettiest scaling algorithm. highly recommend.
//!
//! usage:
//! ```
//! # use fimg::{Image, scale::Lanczos3};
//! let i = Image::<_, 3>::open("tdata/small_cat.png");
//! let scaled = i.scale::<Lanczos3>(2144, 1424);
//! ```
use crate::Image;

mod algorithms;
pub mod traits;
pub use algorithms::*;

macro_rules! transparent {
    ($n: literal, $name: ident) => {
        impl<T: AsMut<[u8]> + AsRef<[u8]>> Image<T, $n> {
            /// Scale a
            #[doc = stringify!($name)]
            /// image with a given scaling algorithm.
            pub fn scale<A: traits::ScalingAlgorithm>(
                &mut self,
                width: u32,
                height: u32,
            ) -> Image<std::boxed::Box<[u8]>, $n> {
                A::scale_transparent(
                    self.as_mut(),
                    width.try_into().unwrap(),
                    height.try_into().unwrap(),
                )
            }
        }
    };
}

macro_rules! opaque {
    ($n: literal, $name: ident) => {
        impl<T: AsRef<[u8]>> Image<T, $n> {
            /// Scale a
            #[doc = stringify!($name)]
            /// image with a given scaling algorithm.
            pub fn scale<A: traits::ScalingAlgorithm>(
                &self,
                width: u32,
                height: u32,
            ) -> Image<std::boxed::Box<[u8]>, $n> {
                A::scale_opaque(
                    self.as_ref(),
                    width.try_into().unwrap(),
                    height.try_into().unwrap(),
                )
            }
        }
    };
}

opaque!(1, Y);
transparent!(2, YA);
opaque!(3, RGB);
transparent!(4, RGBA);

#[test]
fn test_nearest() {
    let i = Image::<_, 3>::open("tdata/cat.png");
    assert_eq!(
        &*i.scale::<Nearest>(268, 178).buffer,
        &*Image::<_, 3>::open("tdata/small_cat.png").buffer
    );
}
