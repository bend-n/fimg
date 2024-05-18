use super::Basic;
use crate::{pixels::convert::PFrom, scale, term::size::fit, Image};
use core::intrinsics::transmute_unchecked as transmute;
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
    [(); N]: Basic,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.write(f)
    }
}

impl<T: AsRef<[u8]>, const N: usize> Debug for Bloc<T, N>
where
    [(); N]: Basic,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.write(f)
    }
}

impl<T: AsRef<[u8]>, const N: usize> Bloc<T, N>
where
    [(); N]: Basic,
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
        let buf;
        let i = if !cfg!(test) {
            let (w, h) = fit((self.width(), self.height()));
            macro_rules! n {
                ($n:literal) => {
                    transmute::<Image<Box<[u8]>, $n>, Image<Box<[u8]>, N>>(
                        transmute::<Image<&[u8], N>, Image<&[u8], $n>>(self.as_ref())
                            .scale::<scale::Nearest>(w, h),
                    )
                };
                (o $n:literal) => {
                    transmute::<Image<Box<[u8]>, 1>, Image<Box<[u8]>, N>>(
                        transmute::<Image<Vec<u8>, N>, Image<Vec<u8>, 1>>(self.as_ref().to_owned())
                            .scale::<scale::Nearest>(w, h),
                    )
                };
            }
            // SAFETY: #[allow(clippy::undocumented_unsafe_blocks)]
            buf = unsafe {
                match N {
                    1 => n![1],
                    2 => n![o 2],
                    3 => n![3],
                    4 => n![o 4],
                    _ => unreachable!(),
                }
            };
            buf.as_ref()
        } else {
            self.as_ref()
        };

        for [a, b] in i
            .flatten()
            .chunks_exact(i.width() as _)
            .map(|x| {
                #[allow(clippy::undocumented_unsafe_blocks)]
                x.iter().copied().map(|x| unsafe {
                    match N {
                        1 => <[u8; 3] as PFrom<1>>::pfrom(transmute(x)),
                        2 => <[u8; 3] as PFrom<2>>::pfrom(transmute(x)),
                        3 => <[u8; 3] as PFrom<3>>::pfrom(transmute(x)),
                        4 => <[u8; 3] as PFrom<4>>::pfrom(transmute(x)),
                        _ => unreachable!(),
                    }
                })
            })
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
