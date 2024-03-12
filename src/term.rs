//! terminal outputs
//! produces output for any terminal supporting one of the
//! ```text
//! Kitty Graphics Protocol
//! Iterm2 Inline Image Protocol
//! Sixel Bitmap Graphics Format
//! ```
//! with a fallback for dumb terminals.
//!
//! the (second?) best way to debug your images.
mod bloc;
mod kitty;
mod sixel;
mod size;
pub use bloc::Bloc;
pub use iterm2::Iterm2;
pub use kitty::Kitty;
pub use sixel::Sixel;
use std::fmt::{Result, Write};

use crate::{pixels::convert::PFrom, Image, WritePng};

mod b64;
mod iterm2;

impl<'a, const N: usize> std::fmt::Display for Image<&'a [u8], N>
where
    [u8; 3]: PFrom<N>,
    [u8; 4]: PFrom<N>,
    Image<&'a [u8], N>: kitty::Data + WritePng,
    Image<&'a [u8], N>: bloc::Scaled<N>,
{
    /// Display an image in the terminal.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result {
        Display(*self).write(f)
    }
}

/// Print an image in the terminal.
///
/// This is a wrapper for `print!("{}", term::Display(image))`
pub fn print<'a, const N: usize>(i: Image<&'a [u8], N>)
where
    [u8; 3]: PFrom<N>,
    [u8; 4]: PFrom<N>,
    Image<&'a [u8], N>: bloc::Scaled<N>,
    Image<&'a [u8], N>: kitty::Data + WritePng,
{
    print!("{}", Display(i))
}

#[derive(Copy, Clone)]
/// Display an image in the terminal.
/// This type implements [`Display`](std::fmt::Display) and [`Debug`](std::fmt::Debug).
pub struct Display<'a, const N: usize>(pub Image<&'a [u8], N>);

impl<'a, const N: usize> std::ops::Deref for Display<'a, N> {
    type Target = Image<&'a [u8], N>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, const N: usize> std::fmt::Debug for Display<'a, N>
where
    [u8; 3]: PFrom<N>,
    [u8; 4]: PFrom<N>,
    Image<&'a [u8], N>: bloc::Scaled<N>,
    Image<&'a [u8], N>: kitty::Data + WritePng,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result {
        self.write(f)
    }
}

impl<'a, const N: usize> std::fmt::Display for Display<'a, N>
where
    Image<&'a [u8], N>: bloc::Scaled<N>,
    [u8; 4]: PFrom<N>,
    [u8; 3]: PFrom<N>,
    Image<&'a [u8], N>: kitty::Data + WritePng,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.write(f)
    }
}

impl<'a, const N: usize> Display<'a, N>
where
    [u8; 4]: PFrom<N>,
    [u8; 3]: PFrom<N>,
    Image<&'a [u8], N>: bloc::Scaled<N>,
    Image<&'a [u8], N>: kitty::Data + WritePng,
{
    /// Write $TERM protocol encoded image data.
    pub fn write(self, f: &mut impl Write) -> Result {
        if let Ok(term) = std::env::var("TERM") {
            match &*term {
                "mlterm" | "yaft-256color" => return Sixel(*self).write(f),
                x if x.contains("kitty") => return Kitty(*self).write(f),
                _ => (),
            }
        }
        if let Ok(term_program) = std::env::var("TERM_PROGRAM") {
            match &*term_program {
                "MacTerm" => return Sixel(*self).write(f),
                "iTerm" | "WezTerm" => return Iterm2(*self).write(f),
                _ => (),
            }
        }
        if let Ok("iTerm") = std::env::var("LC_TERMINAL").as_deref() {
            return Iterm2(*self).write(f);
        }
        #[cfg(unix)]
        return self.guess_harder(f).unwrap_or_else(|| Bloc(*self).write(f));
        #[cfg(not(unix))]
        return Bloc(*self).write(f);
    }

    #[cfg(unix)]
    // https://github.com/benjajaja/ratatui-image/blob/master/src/picker.rs#L226
    fn guess_harder(self, to: &mut impl Write) -> Option<Result> {
        extern crate libc;
        use std::{io::Read, mem::MaybeUninit};

        fn r(result: i32) -> Option<()> {
            (result != -1).then_some(())
        }

        let mut termios = MaybeUninit::<libc::termios>::uninit();
        // SAFETY: get termios of stdin
        r(unsafe { libc::tcgetattr(0, termios.as_mut_ptr()) })?;
        // SAFETY: gotten
        let termios = unsafe { termios.assume_init() };

        // SAFETY: turn off echo and canonical (requires enter before stdin reads) modes
        unsafe {
            libc::tcsetattr(
                0,
                libc::TCSADRAIN,
                &libc::termios {
                    c_lflag: termios.c_lflag & !libc::ICANON & !libc::ECHO,
                    ..termios
                },
            )
        };
        let buf = {
            // contains a kitty gfx and sixel query, the `\x1b[c` is for sixels
            println!(r"_Gi=31,s=1,v=1,a=q,t=d,f=24;AAAA\[c");
            let mut stdin = std::io::stdin();
            let mut buf = String::new();

            let mut b = [0; 16];
            'l: loop {
                let n = stdin.read(&mut b).ok()?;
                if n == 0 {
                    continue;
                }
                for b in b {
                    buf.push(b as char);
                    if b == b'c' {
                        break 'l;
                    }
                }
            }
            buf
        };

        // SAFETY: reset attrs to what they were before we became nosy
        unsafe { libc::tcsetattr(0, libc::TCSADRAIN, &termios) };

        if buf.contains("_Gi=31;OK") {
            Some(Kitty(*self).write(to))
        } else if buf.contains(";4;")
            || buf.contains("?4;")
            || buf.contains(";4c")
            || buf.contains("?4c")
        {
            Some(Sixel(*self).write(to))
        } else {
            None
        }
    }
}
