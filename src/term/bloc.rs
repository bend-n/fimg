use crate::{pixels::convert::PFrom, Image};
use std::fmt::{Debug, Display, Formatter, Result, Write};

/// Colored `▀`s. The simple, stupid solution.
/// May be too big for your terminal.
pub struct Bloc<T: AsRef<[u8]>, const N: usize>(pub Image<T, N>);
impl<T: AsRef<[u8]>, const N: usize> std::ops::Deref for Bloc<T, N> {
    type Target = Image<T, N>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: AsRef<[u8]>, const N: usize> Display for Bloc<T, N>
where
    [u8; 3]: PFrom<N>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.write(f)
    }
}

impl<T: AsRef<[u8]>, const N: usize> Debug for Bloc<T, N>
where
    [u8; 3]: PFrom<N>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.write(f)
    }
}

impl<T: AsRef<[u8]>, const N: usize> Bloc<T, N>
where
    [u8; 3]: PFrom<N>,
{
    /// Write out halfblocks.
    pub fn write(&self, to: &mut impl Write) -> Result {
        macro_rules! c {
            (fg $fg:expr, bg $bg:expr) => {{
                let [fr, fg, fb] = $fg;
                let [br, bg, bb] = $bg;
                write!(to, "\x1b[38;2;{fr};{fg};{fb};48;2;{br};{bg};{bb}m▀")?;
            }};
        }
        // TODO: scale 2 fit
        for [a, b] in self
            .flatten()
            .chunks_exact(self.width() as _)
            .map(|x| x.iter().copied().map(<[u8; 3] as PFrom<N>>::pfrom))
            .array_chunks::<2>()
        {
            for (a, b) in a.zip(b) {
                c! { fg a, bg b };
            }
            writeln!(to)?;
        }
        write!(to, "\x1b[0m")?;
        Ok(())
    }
}

#[test]
fn test() {
    let x = Image::<_, 3>::open("tdata/small_cat.png");
    use std::hash::Hasher;
    let mut h = std::hash::DefaultHasher::new();
    h.write(Bloc(x).to_string().as_bytes());
    assert_eq!(h.finish(), 0x6546104ffee16f77);
}
