use super::b64;
use crate::Image;
use std::borrow::Cow;
use std::fmt::{Debug, Display, Formatter, Result, Write};

/// Outputs [Kitty Graphics Protocol](https://sw.kovidgoyal.net/kitty/graphics-protocol) encoded data.
pub struct Kitty<T: AsRef<[u8]>, const N: usize>(pub Image<T, N>);

impl<T: AsRef<[u8]>, const N: usize> std::ops::Deref for Kitty<T, N> {
    type Target = Image<T, N>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: AsRef<[u8]>, const N: usize> Display for Kitty<T, N>
where
    Image<T, N>: Data,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.write(f)
    }
}

impl<T: AsRef<[u8]>, const N: usize> Debug for Kitty<T, N>
where
    Image<T, N>: Data,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.write(f)
    }
}

mod seal {
    pub trait Sealed {}
}
use seal::Sealed;

pub trait Data: Sealed {
    #[doc(hidden)]
    fn get(&self) -> (Cow<[u8]>, &'static str);
}
macro_rules! imp {
    ($n:literal, $f:expr) => {
        impl<T: AsRef<[u8]>> Sealed for Image<T, $n> {}
        impl<T: AsRef<[u8]>> Data for Image<T, $n> {
            fn get(&self) -> (Cow<[u8]>, &'static str) {
                const fn castor<
                    T: AsRef<[u8]>,
                    F: FnMut(&Image<T, $n>) -> (Cow<[u8]>, &'static str),
                >(
                    f: F,
                ) -> F {
                    f
                }
                castor($f)(self)
            }
        }
    };
}
imp! { 4, |x| (Cow::from(x.bytes()), "32") }
imp! { 3, |x| (Cow::from(x.bytes()), "24") }
imp! { 2, |x| (Cow::Owned(<Image<Box<[u8]>, 3>>::from(x.as_ref()).take_buffer().to_vec()), "24") }
imp! { 1, |x| (Cow::Owned(<Image<Box<[u8]>, 3>>::from(x.as_ref()).take_buffer().to_vec()), "24") }

impl<T: AsRef<[u8]>, const N: usize> Kitty<T, N> {
    /// Write out kitty gfx data.
    pub fn write(&self, to: &mut impl Write) -> Result
    where
        Image<T, N>: Data,
    {
        let (bytes, dtype) = self.get();
        let (w, h) = (self.width(), self.height());

        let mut enc = Vec::with_capacity(b64::size(&bytes));
        b64::encode(&bytes, &mut enc).unwrap();
        let mut chunks = enc
            .chunks(4096)
            // SAFETY: b64
            .map(|x| unsafe { std::str::from_utf8_unchecked(x) });

        let last = chunks.len();
        const H: &str = "_G";
        const MORE: &str = "m";

        const DISPLAY: char = 'T';
        const ACTION: char = 'a';
        const DATATYPE: char = 'f';
        const TYPE: char = 't';
        const DIRECT: char = 'd';

        const E: &str = r"\";

        let payload = chunks.next().unwrap();
        let more = (last > 1) as u8;
        write!(to, "{H}{DATATYPE}={dtype},{ACTION}={DISPLAY},{TYPE}={DIRECT},s={w},v={h},{MORE}={more};{payload}{E}")?;

        for (payload, i) in chunks.zip(2..) {
            let more = (i != last) as u8;
            write!(to, "{H}{MORE}={more};{payload}{E}")?;
        }
        Ok(())
    }
}

#[test]
fn test() {
    let x = Image::<_, 3>::open("tdata/cat.png");
    use std::hash::Hasher;
    let mut h = std::hash::DefaultHasher::new();
    h.write(Kitty(x).to_string().as_bytes());
    assert_eq!(h.finish(), 0x1cc13114bcf3cc3);
}
