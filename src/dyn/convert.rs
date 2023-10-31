#![allow(clippy::useless_conversion)]
use super::{e, DynImage, Image};

impl From<DynImage<Box<[u8]>>> for Image<Box<[u8]>, 1> {
    fn from(value: DynImage<Box<[u8]>>) -> Self {
        e!(value, |i| i.into())
    }
}

impl From<DynImage<Box<[u8]>>> for Image<Box<[u8]>, 2> {
    fn from(value: DynImage<Box<[u8]>>) -> Self {
        e!(value, |i| i.into())
    }
}
impl From<DynImage<Box<[u8]>>> for Image<Box<[u8]>, 3> {
    fn from(value: DynImage<Box<[u8]>>) -> Self {
        e!(value, |i| i.into())
    }
}
impl From<DynImage<Box<[u8]>>> for Image<Box<[u8]>, 4> {
    fn from(value: DynImage<Box<[u8]>>) -> Self {
        e!(value, |i| i.into())
    }
}

impl<T> DynImage<T> {
    /// Gets out the Y image, if its there, else returning self.
    ///
    /// If you want to convert, see [`DynImage::to_y`].
    pub fn get_y(self) -> Result<Image<T, 1>, Self> {
        match self {
            Self::Y(i) => Ok(i),
            _ => Err(self),
        }
    }

    /// Gets out the YA image, if its there, else returning self.
    ///
    /// If you want to convert, see [`DynImage::to_ya`].
    pub fn get_ya(self) -> Result<Image<T, 2>, Self> {
        match self {
            Self::Ya(i) => Ok(i),
            _ => Err(self),
        }
    }

    /// Gets out the RGB image, if its there, else returning self.
    ///
    /// If you want to convert, see [`DynImage::to_rgb`].
    pub fn get_rgb(self) -> Result<Image<T, 3>, Self> {
        match self {
            Self::Rgb(i) => Ok(i),
            _ => Err(self),
        }
    }

    /// Gets out the RGBA image, if its there, else returning self.
    ///
    /// If you want to convert, see [`DynImage::to_rgba`].
    pub fn get_rgba(self) -> Result<Image<T, 4>, Self> {
        match self {
            Self::Rgba(i) => Ok(i),
            _ => Err(self),
        }
    }
}

impl DynImage<Box<[u8]>> {
    /// Convert this dyn image into a Y image.
    pub fn to_y(self) -> Image<Box<[u8]>, 1> {
        self.into()
    }

    /// Convert this dyn image into a YA image.
    pub fn to_ya(self) -> Image<Box<[u8]>, 2> {
        self.into()
    }

    /// Convert this dyn image into a RGB image.
    pub fn to_rgb(self) -> Image<Box<[u8]>, 3> {
        self.into()
    }

    /// Convert this dyn image into a RGBA image.
    pub fn to_rgba(self) -> Image<Box<[u8]>, 4> {
        self.into()
    }
}

impl<T: AsRef<[u8]>> DynImage<T> {
    /// Produce a Y image from this dyn image.
    pub fn y(&self) -> Image<Box<[u8]>, 1> {
        e!(self, |i| i.as_ref().into())
    }

    /// Produce a YA image from this dyn image.
    pub fn ya(&self) -> Image<Box<[u8]>, 2> {
        e!(self, |i| i.as_ref().into())
    }

    /// Produce a RGB image from this dyn image.
    pub fn rgb(&self) -> Image<Box<[u8]>, 3> {
        e!(self, |i| i.as_ref().into())
    }

    /// Produce a RGBA image from this dyn image.
    pub fn rgba(&self) -> Image<Box<[u8]>, 4> {
        e!(self, |i| i.as_ref().into())
    }
}
