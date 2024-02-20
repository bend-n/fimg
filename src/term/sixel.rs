use std::fmt::{Debug, Display, Formatter, Result, Write};

use crate::{pixels::convert::PFrom, Image};

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
    [u8; 4]: PFrom<N>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.write(f)
    }
}

impl<T: AsRef<[u8]>, const N: usize> Debug for Sixel<T, N>
where
    [u8; 4]: PFrom<N>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.write(f)
    }
}

impl<T: AsRef<[u8]>, const N: usize> Sixel<T, N> {
    /// Write out sixel data.
    pub fn write(&self, to: &mut impl Write) -> Result
    where
        [u8; 4]: PFrom<N>,
    {
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
                .map(<[u8; 4] as PFrom<N>>::pfrom)
                .collect::<Vec<_>>();
            &*buf
        };

        let q = qwant::NeuQuant::new(15, 255, rgba);
        // TODO: don't colllect
        let pixels: Vec<u8> = rgba.iter().map(|&pix| q.index_of(pix) as u8).collect();

        for ([r, g, b], i) in q
            .color_map_rgb()
            .map(|x| x.map(|x| (x as f32 * (100. / 255.)) as u32))
            .zip(0u8..)
        {
            write!(to, "#{i};2;{r};{g};{b}")?;
        }
        for sixel_row in pixels.chunks_exact(self.width() as usize * 6).map(|x| {
            let mut x = x
                .iter()
                .zip(0u32..)
                .map(|(&p, j)| (p, (j % self.width(), j / self.width())))
                .collect::<Vec<_>>();
            x.sort_unstable();
            x
        }) {
            // extracted
            for samples in Grouped(&sixel_row, |r| r.0) {
                write!(to, "#{}", samples[0].0)?;
                let mut last = -1;
                for (x, byte) in Grouped(samples, |(_, (x, _))| x).map(|v| {
                    (
                        v[0].1 .0 as i32,
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
