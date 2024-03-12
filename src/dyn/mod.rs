use crate::{pixels::convert::PFrom, Image};
mod affine;
mod convert;
#[cfg(feature = "scale")]
mod scale;

#[derive(Clone, Debug, PartialEq)]
/// Dynamic image.
/// Can be any of {`Y8`, `YA8`, `RGB8`, `RGB16`}.
pub enum DynImage<T> {
    /// Y image
    Y(Image<T, 1>),
    /// YA image
    Ya(Image<T, 2>),
    /// RGB image
    Rgb(Image<T, 3>),
    /// RGBA image
    Rgba(Image<T, 4>),
}

impl Copy for DynImage<&[u8]> {}

macro_rules! e {
    ($dyn:expr => |$image: pat_param| $do:expr) => {
        match $dyn {
            DynImage::Y($image) => DynImage::Y($do),
            DynImage::Ya($image) => DynImage::Ya($do),
            DynImage::Rgb($image) => DynImage::Rgb($do),
            DynImage::Rgba($image) => DynImage::Rgba($do),
        }
    };
    ($dyn:expr, |$image: pat_param| $do:expr) => {
        match $dyn {
            DynImage::Y($image) => $do,
            DynImage::Ya($image) => $do,
            DynImage::Rgb($image) => $do,
            DynImage::Rgba($image) => $do,
        }
    };
}
use e;

#[cfg(feature = "term")]
impl<T: AsRef<[u8]>> std::fmt::Display for crate::term::Display<DynImage<T>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        e!(&self.0, |x| crate::term::Display(x.as_ref()).write(f))
    }
}

#[cfg(feature = "term")]
impl<T: AsRef<[u8]>> std::fmt::Debug for crate::term::Display<DynImage<T>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        e!(&self.0, |x| crate::term::Display(x.as_ref()).write(f))
    }
}

#[cfg(feature = "term")]
impl<T: AsRef<[u8]>> std::fmt::Display for DynImage<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        e!(&self, |x| crate::term::Display(x.as_ref()).write(f))
    }
}

impl<T> DynImage<T> {
    /// Get the width of this image.
    pub const fn width(&self) -> u32 {
        e!(self, |i| i.width())
    }

    /// Get the height of this image.
    pub const fn height(&self) -> u32 {
        e!(self, |i| i.height())
    }

    /// Get the image buffer.
    pub const fn buffer(&self) -> &T {
        e!(self, |i| i.buffer())
    }

    /// Take the image buffer.
    pub fn take_buffer(self) -> T {
        e!(self, |i| i.take_buffer())
    }
}

impl<T: AsRef<[u8]>> DynImage<T> {
    /// Reference this image.
    pub fn as_ref(&self) -> DynImage<&[u8]> {
        e!(self => |i| i.as_ref())
    }

    /// Get a pixel, of a type.
    /// ```
    /// # use fimg::{Image, DynImage};
    /// let i = DynImage::Rgb(Image::alloc(50, 50));
    /// assert_eq!(unsafe { i.pixel::<4>(25, 25) }, [0, 0, 0, 255]);
    /// ```
    /// # Safety
    ///
    /// undefined behaviour if pixel is out of bounds.
    pub unsafe fn pixel<const P: usize>(&self, x: u32, y: u32) -> [u8; P]
    where
        [u8; P]: PFrom<1>,
        [u8; P]: PFrom<2>,
        [u8; P]: PFrom<3>,
        [u8; P]: PFrom<4>,
    {
        e!(self, |i| PFrom::pfrom(unsafe { i.pixel(x, y) }))
    }

    /// Bytes of this image.
    pub fn bytes(&self) -> &[u8] {
        e!(self, |i| i.bytes())
    }
}

impl DynImage<Box<[u8]>> {
    #[cfg(feature = "save")]
    /// Open a PNG image
    pub fn open(f: impl AsRef<std::path::Path>) -> Self {
        use png::Transformations as T;
        let p = std::fs::File::open(f).unwrap();
        let r = std::io::BufReader::new(p);
        let mut dec = png::Decoder::new(r);
        dec.set_transformations(T::STRIP_16 | T::EXPAND);
        let mut reader = dec.read_info().unwrap();
        let mut buf = vec![0; reader.output_buffer_size()].into_boxed_slice();
        let info = reader.next_frame(&mut buf).unwrap();
        use png::ColorType::*;
        match info.color_type {
            Indexed | Grayscale => Self::Y(Image::build(info.width, info.height).buf(buf)),
            Rgb => Self::Rgb(Image::build(info.width, info.height).buf(buf)),
            Rgba => Self::Rgba(Image::build(info.width, info.height).buf(buf)),
            GrayscaleAlpha => Self::Ya(Image::build(info.width, info.height).buf(buf)),
        }
    }

    #[cfg(feature = "save")]
    /// Save this image to a PNG.
    pub fn save(&self, f: impl AsRef<std::path::Path>) {
        let p = std::fs::File::create(f).unwrap();
        let w = &mut std::io::BufWriter::new(p);
        let mut enc = png::Encoder::new(w, self.width(), self.height());
        enc.set_depth(png::BitDepth::Eight);
        enc.set_color(match self {
            Self::Y(_) => png::ColorType::Grayscale,
            Self::Ya(_) => png::ColorType::GrayscaleAlpha,
            Self::Rgb(_) => png::ColorType::Rgb,
            Self::Rgba(_) => png::ColorType::Rgba,
        });
        enc.set_source_gamma(png::ScaledFloat::new(1.0 / 2.2));
        enc.set_source_chromaticities(png::SourceChromaticities::new(
            (0.31270, 0.32900),
            (0.64000, 0.33000),
            (0.30000, 0.60000),
            (0.15000, 0.06000),
        ));
        let mut writer = enc.write_header().unwrap();
        writer.write_image_data(self.bytes()).unwrap();
    }
}
