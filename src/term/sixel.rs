use core::intrinsics::transmute_unchecked as transmute;
use std::fmt::{Debug, Display, Formatter, Result, Write};

use crate::{Image, pixels::convert::PFrom};

use super::Basic;

/// Outputs [sixel](https://en.wikipedia.org/wiki/Sixel) encoded data in its [`Display`] and [`Debug`] implementations, for easy visual debugging.
pub struct Sixel<T: AsRef<[u8]>, const N: usize>(pub Image<T, N>);

impl<T: AsRef<[u8]>, const N: usize> std::ops::Deref for Sixel<T, N> {
    type Target = Image<T, N>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: AsRef<[u8]>, const N: usize> Display for Sixel<T, N>
where
    [(); N]: Basic,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.write(f)
    }
}

impl<T: AsRef<[u8]>, const N: usize> Debug for Sixel<T, N>
where
    [(); N]: Basic,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.write(f)
    }
}

impl<T: AsRef<[u8]>, const N: usize> Sixel<T, N> {
    /// Write out sixel data.
    pub fn write(&self, to: &mut impl Write) -> Result
    where
        [(); N]: Basic,
    {
        #[cfg(unix)]
        let q = {
            extern crate libc;
            // SAFETY: is stdout a tty
            (unsafe { libc::isatty(1) } == 1)
        };
        #[cfg(not(unix))]
        let q = true;
        let colors = q
            .then(|| {
                super::query("[?1;1;0S").and_then(|x| {
                    // [?1;0;65536S
                    if let [b'?', b'1', b';', b'0', b';', n @ ..] = x.as_bytes() {
                        Some(
                            n.iter()
                                .copied()
                                .take_while(u8::is_ascii_digit)
                                .fold(0u16, |acc, x| {
                                    acc.saturating_mul(10).saturating_add((x - b'0') as u16)
                                })
                                .max(64)
                                .min(0xfff),
                        )
                    } else {
                        None
                    }
                })
            })
            .flatten()
            .unwrap_or(255);
        to.write_str("Pq")?;
        write!(to, r#""1;1;{};{}"#, self.width(), self.height())?;
        let buf;
        let rgba = if N == 4 {
            // SAFETY: buffer cannot have half pixels (cant use flatten bcoz N)
            unsafe { self.buffer().as_ref().as_chunks_unchecked() }
        } else {
            buf = self
                .chunked()
                .copied()
                // SAFETY: #[allow(clippy::undocumented_unsafe_blocks)]
                .map(|x| unsafe {
                    match N {
                        1 => <[u8; 4] as PFrom<1>>::pfrom(transmute(x)),
                        2 => <[u8; 4] as PFrom<2>>::pfrom(transmute(x)),
                        3 => <[u8; 4] as PFrom<3>>::pfrom(transmute(x)),
                        _ => unreachable!(),
                    }
                })
                .collect::<Vec<_>>();
            &*buf
        };

        let q = qwant::NeuQuant::new(15, colors as _, rgba);

        // TODO: don't colllect
        let pixels: Vec<u16> = rgba.iter().map(|&pix| q.index_of(pix) as _).collect();

        for ([r, g, b], i) in q
            .color_map_rgb()
            .map(|x| x.map(|x| (x as f32 * (100. / 255.)) as u32))
            .zip(0u64..)
        {
            write!(to, "#{i};2;{r};{g};{b}")?;
        }
        for sixel_row in pixels.chunks_exact(self.width() as usize * 6).map(|x| {
            x.iter()
                .zip(0u32..)
                .map(|(&p, j)| (p, (j % self.width(), j / self.width())))
                .collect::<Vec<_>>()
        }) {
            // extracted
            for samples in Grouped(&sixel_row, |r| r.0) {
                write!(to, "#{}", samples[0].0)?;
                let mut last = -1;
                for (x, byte) in Grouped(samples, |(_, (x, _))| x).map(|v| {
                    (
                        v[0].1.0 as i32,
                        v.iter()
                            .map(|&(_, (_, y))| (1 << y))
                            .fold(0, |acc, x| acc | x),
                    )
                }) {
                    if last + 1 != x {
                        write!(to, "!{}?", x - last - 1)?;
                    }
                    to.write_char((byte + b'?') as char)?;
                    last = x;
                }

                write!(to, "$")?;
            }
            write!(to, "-")?;
        }
        write!(to, r"\")?;

        Ok(())
    }
}

struct Grouped<'a, K: Eq, T, F: Fn(T) -> K>(&'a [T], F);
impl<'a, K: Eq, T: Copy, F: Fn(T) -> K> Iterator for Grouped<'a, K, T, F> {
    type Item = &'a [T];
    fn next(&mut self) -> Option<Self::Item> {
        self.0.first()?;
        self.0
            .split_at_checked(
                self.0
                    .array_windows::<2>()
                    .take_while(|&&[a, b]| (self.1)(a) == (self.1)(b))
                    .count()
                    + 1,
            )
            .inspect(|(_, t)| self.0 = t)
            .map(|(h, _)| h)
    }
}

#[test]
fn test() {
    assert_eq!(
        Sixel(Image::<Vec<u8>, 3>::open("tdata/small_cat.png")).to_string(),
        include_str!("../../tdata/small_cat.six")
    );
}
