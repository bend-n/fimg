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
use crate::Image;
pub use bloc::Bloc;
pub use iterm2::Iterm2;
pub use kitty::Kitty;
pub use sixel::Sixel;
use std::fmt::{Result, Write};

mod seal {
    pub trait Sealed {}
}
use seal::Sealed;
#[doc(hidden)]
pub trait Basic: Sealed {}
impl Sealed for [(); 1] {}
impl Basic for [(); 1] {}
impl Sealed for [(); 2] {}
impl Basic for [(); 2] {}
impl Sealed for [(); 3] {}
impl Basic for [(); 3] {}
impl Sealed for [(); 4] {}
impl Basic for [(); 4] {}

mod b64;
mod iterm2;

impl<'a, const N: usize> std::fmt::Display for Image<&'a [u8], N>
where
    [(); N]: Basic,
{
    /// Display an image in the terminal.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result {
        Display(*self).write(f)
    }
}

/// Print an image in the terminal.
///
/// This is a wrapper for `print!("{}", term::Display(image))`
pub fn print<T: AsRef<[u8]>, const N: usize>(i: Image<T, N>)
where
    [(); N]: Basic,
    Display<Image<T, N>>: std::fmt::Display,
{
    print!("{}", Display(i))
}

#[derive(Copy, Clone)]
/// Display an image in the terminal.
/// This type implements [`Display`](std::fmt::Display) and [`Debug`](std::fmt::Debug).
pub struct Display<T>(pub T);

impl<T> std::ops::Deref for Display<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: AsRef<[u8]>, const N: usize> std::fmt::Debug for Display<Image<T, N>>
where
    [(); N]: Basic,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result {
        Display(self.as_ref()).write(f)
    }
}

impl<const N: usize> Display<Image<&[u8], N>>
where
    [(); N]: Basic,
{
    /// Write $TERM protocol encoded image data.
    pub fn write(self, f: &mut impl Write) -> Result {
        if let Ok(term) = std::env::var("TERM") {
            match &*term {
                "mlterm" | "yaft-256color" => return Sixel(self.0).write(f),
                x if x.contains("kitty") => return Kitty(self.0).write(f),
                _ => (),
            }
        }
        if let Ok(term_program) = std::env::var("TERM_PROGRAM") {
            match &*term_program {
                "MacTerm" => return Sixel(self.0).write(f),
                "iTerm" | "WezTerm" => return Iterm2(self.0).write(f),
                _ => (),
            }
        }
        if let Ok("iTerm") = std::env::var("LC_TERMINAL").as_deref() {
            return Iterm2(self.0).write(f);
        }
        #[cfg(unix)]
        return self
            .guess_harder(f)
            .unwrap_or_else(|| Bloc(self.0).write(f));
        #[cfg(not(unix))]
        return Bloc(*self).write(f);
    }

    #[cfg(unix)]
    // https://github.com/benjajaja/ratatui-image/blob/master/src/picker.rs#L226
    fn guess_harder(&self, to: &mut impl Write) -> Option<Result> {
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
            Some(Kitty(self.as_ref()).write(to))
        } else if buf.contains(";4;")
            || buf.contains("?4;")
            || buf.contains(";4c")
            || buf.contains("?4c")
        {
            Some(Sixel(self.as_ref()).write(to))
        } else {
            None
        }
    }
}
