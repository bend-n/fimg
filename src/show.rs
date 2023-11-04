use crate::Image;

#[cfg(feature = "real-show")]
mod real {
    use crate::Image;
    use minifb::{Key, Window};

    pub fn show(i: Image<&[u32], 1>) {
        let mut win = Window::new(
            "show",
            i.width() as usize,
            i.height() as usize,
            Default::default(),
        )
        .unwrap();
        win.limit_update_rate(Some(std::time::Duration::from_millis(100)));
        while win.is_open() && !win.is_key_down(Key::Q) && !win.is_key_down(Key::Escape) {
            win.update_with_buffer(&i.buffer, i.width() as usize, i.height() as usize)
                .expect("window update fail");
        }
    }
}

#[cfg(not(feature = "real-show"))]
mod fake {
    use std::process::{Command, Stdio};

    macro_rules! c {
        ($p:literal) => {
            std::process::Command::new($p)
        };
        ($p:literal $($args:expr)+) => {
            std::process::Command::new($p).args([$($args,)+])
        }
    }
    pub(crate) use c;

    pub fn has(c: &'static str) -> bool {
        complete(c!("which").arg(c))
    }

    pub fn complete(c: &mut Command) -> bool {
        c.stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .expect("ok")
            .success()
    }

    macro_rules! show {
        ($me:expr) => {
            let file = std::env::temp_dir().join("viewing.png");
            $me.save(&file);
            #[cfg(target_family = "windows")]
            assert!(fake::complete(fake::c!("start" "%Temp%/viewing.png")), "command should complete successfully.");
            #[cfg(target_family = "unix")]
            assert!(
            if fake::has("feh") { fake::complete(fake::c!("feh" file))
            } else if fake::has("xdg-open") { fake::complete(fake::c!("xdg-open" file))
            } else if fake::has("gio") { fake::complete(fake::c!("gio" file))
            } else if fake::has("gnome-open") { fake::complete(fake::c!("gnome-open" file))
            } else if fake::has("kde-open") { fake::complete(fake::c!("kde-open" file))
            } else if fake::has("open") { fake::complete(fake::c!("open" file))
            } else { panic!("no image viewer found, please enable the `real-show` feature.") },
            "command should complete successfully.");
        };

    }
    pub(crate) use show;
}

fn r(i: &Image<Box<[u32]>, 1>) -> Image<&[u32], 1> {
    // SAFETY: ctor
    unsafe { Image::new(i.width, i.height, &*i.buffer) }
}

macro_rules! show {
    ($buf:ty) => {
        show!($buf, 1);
        show!($buf, 2);
        show!($buf, 3);
        show!($buf, 4);
    };
    ($buf:ty, $n:literal) => {
        impl Image<$buf, $n> {
            /// Open a window showing this image.
            /// Blocks until the window finishes.
            ///
            /// This is like [`dbg!`] for images.
            ///
            /// # Panics
            ///
            /// if the window is un creatable
            pub fn show(self) -> Self {
                #[cfg(feature = "real-show")]
                real::show(r(&self.as_ref().into()));
                #[cfg(not(feature = "real-show"))]
                fake::show!(self);
                self
            }
        }
    };
}

show!(Vec<u8>);
show!(Box<[u8]>);
show!(&[u8]);

impl Image<Box<[u32]>, 1> {
    /// Open a window showing this image.
    /// Blocks until the window finishes.
    ///
    /// This is like [`dbg!`] for images.
    ///
    /// # Panics
    ///
    /// if the window is un creatable
    pub fn show(self) -> Self {
        #[cfg(feature = "real-show")]
        real::show(r(&self));
        #[cfg(not(feature = "real-show"))]
        fake::show!(Image::<Box<[u8]>, 4>::from(r(&self)));
        self
    }
}
