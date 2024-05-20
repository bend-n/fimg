use super::{b64, Basic};
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
    [(); N]: Basic,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.write(f)
    }
}

impl<T: AsRef<[u8]>, const N: usize> Debug for Kitty<T, N>
where
    [(); N]: Basic,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.write(f)
    }
}

impl<T: AsRef<[u8]>, const N: usize> Kitty<T, N> {
    /// Write out kitty gfx data.
    pub fn write(&self, to: &mut impl Write) -> Result
    where
        [(); N]: Basic,
    {
        macro_rules! cast {
            ($n:literal) => {
                (
                    Cow::Owned(
                        <Image<Box<[u8]>, 3>>::from({
                            // SAFETY: ...
                            unsafe { self.as_ref().trans::<$n>() }
                        })
                        .take_buffer()
                        .to_vec(),
                    ),
                    "24",
                )
            };
        }
        let (bytes, dtype) = {
            match N {
                1 => cast!(1),
                2 => cast!(2),
                3 => (Cow::from(self.bytes()), "24"),
                4 => (Cow::from(self.bytes()), "32"),
                _ => unreachable!(),
            }
        };
        let (w, h) = (self.width(), self.height());

        let enc = b64::encode(&bytes);
        let mut chunks = enc
            .as_bytes()
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
