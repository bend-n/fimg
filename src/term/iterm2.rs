use super::b64;
use crate::{Image, WritePng};
use std::fmt::{Debug, Display, Formatter, Result, Write};

/// Outputs [Iterm2 Inline image protocol](https://iterm2.com/documentation-images.html) encoded data.
pub struct Iterm2<T: AsRef<[u8]>, const N: usize>(pub Image<T, N>);
impl<T: AsRef<[u8]>, const N: usize> std::ops::Deref for Iterm2<T, N> {
    type Target = Image<T, N>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: AsRef<[u8]>, const N: usize> Display for Iterm2<T, N>
where
    Image<T, N>: WritePng,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.write(f)
    }
}

impl<T: AsRef<[u8]>, const N: usize> Debug for Iterm2<T, N>
where
    Image<T, N>: WritePng,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.write(f)
    }
}

impl<T: AsRef<[u8]>, const N: usize> Iterm2<T, N>
where
    Image<T, N>: WritePng,
{
    /// Write out kitty gfx data.
    pub fn write(&self, to: &mut impl Write) -> Result {
        let mut d = Vec::with_capacity(1024);
        WritePng::write(&**self, &mut d).unwrap();
        let mut e = Vec::with_capacity(b64::size(&d));
        b64::encode(&d, &mut e).unwrap();
        writeln!(
            to,
            "]1337;File=inline=1;preserveAspectRatio=1;size={}:{}",
            d.len(),
            // SAFETY: b64
            unsafe { std::str::from_utf8_unchecked(&e) }
        )?;
        Ok(())
    }
}

#[test]
fn test() {
    let x = Image::<_, 3>::open("tdata/small_cat.png");
    use std::hash::Hasher;
    let mut h = std::hash::DefaultHasher::new();
    h.write(Iterm2(x).to_string().as_bytes());
    assert_eq!(h.finish(), 0x32e81fb3cea8336f);
}
