use super::{b64, Basic};
use crate::{Image, WritePng};
use core::intrinsics::transmute_unchecked as transmute;
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
    [(); N]: Basic,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.write(f)
    }
}

impl<T: AsRef<[u8]>, const N: usize> Debug for Iterm2<T, N>
where
    [(); N]: Basic,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.write(f)
    }
}

impl<T: AsRef<[u8]>, const N: usize> Iterm2<T, N>
where
    [(); N]: Basic,
{
    /// Write out kitty gfx data.
    pub fn write(&self, to: &mut impl Write) -> Result {
        let mut d = Vec::with_capacity(1024);
        macro_rules! n {
            ($n:literal) => {
                WritePng::write(
                    // SAFETY: ... i renounce traits
                    &unsafe { self.as_ref().trans::<$n>() },
                    &mut d,
                )
                .unwrap()
            };
        }
        match N {
            1 => n![1],
            2 => n![2],
            3 => n![3],
            4 => n![4],
            _ => unreachable!(),
        }
        let e = b64::encode(&d);
        writeln!(
            to,
            "]1337;File=inline=1;preserveAspectRatio=1;size={}:{e}",
            d.len(),
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
