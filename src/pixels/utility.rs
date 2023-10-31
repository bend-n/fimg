use umath::FF32;
pub trait Unfloatify<const N: usize> {
    /// computes 255 * n, for all elements
    fn unfloat(self) -> [u8; N];
}

#[inline(always)]
/// computes 255 * n
pub fn unfloat(n: FF32) -> u8 {
    // SAFETY: n is 0..=1
    unsafe { *(FF32::new(255.0) * n) as u8 }
}

impl<const N: usize> Unfloatify<N> for [FF32; N] {
    fn unfloat(self) -> [u8; N] {
        self.map(unfloat)
    }
}

#[rustfmt::skip]
impl<const N:usize>Unfloatify<N>for[u8; N]{fn unfloat(self)->[u8;N]{self}}

pub trait Floatify<const N: usize> {
    /// computes n / 255, for all elements
    fn float(self) -> [FF32; N];
}

/// computes n / 255
pub fn float(n: u8) -> FF32 {
    // SAFETY: 0..=255 / 0..=255 maynt ever be NAN / INF
    unsafe { FF32::new(n as f32) / FF32::new(255.0) }
}

impl<const N: usize> Floatify<N> for [u8; N] {
    fn float(self) -> [FF32; N] {
        self.map(float)
    }
}

#[rustfmt::skip]
impl<const N:usize>Floatify<N>for[FF32;N]{fn float(self)->[FF32;N]{self}}

pub trait PMap<T, R, const N: usize> {
    /// think of it like a `a.zip(b).map(f).collect::<[]>()`
    fn pmap(self, with: Self, f: impl FnMut(T, T) -> R) -> [R; N];
}

impl<const N: usize, T: Copy, R: Copy> PMap<T, R, N> for [T; N] {
    fn pmap(self, with: Self, mut f: impl FnMut(T, T) -> R) -> [R; N] {
        let mut iter = self.into_iter().zip(with).map(|(a, b)| f(a, b));
        std::array::from_fn(|_| iter.next().unwrap())
    }
}

pub trait Trunc<T, const N: usize> {
    /// it does `a[..a.len() - 1].try_into().unwrap()`.
    fn trunc(&self) -> [T; N - 1];
}

impl<const N: usize, T: Copy> Trunc<T, N> for [T; N] {
    fn trunc(&self) -> [T; N - 1] {
        self[..N - 1].try_into().unwrap()
    }
}

pub trait Push<T, const N: usize> {
    fn and(self, and: T) -> [T; N + 1];
}

impl<const N: usize, T> Push<T, N> for [T; N] {
    fn and(self, and: T) -> [T; N + 1] {
        let mut iter = self.into_iter().chain(std::iter::once(and));
        std::array::from_fn(|_| iter.next().unwrap())
    }
}

#[test]
fn trunc() {
    let x = [1];
    assert_eq!(x.trunc(), []);
    let x = [1, 2, 3, 4];
    assert_eq!(x.trunc(), [1, 2, 3]);
}

#[test]
fn append() {
    let x = [1];
    assert_eq!(x.and(5), [1, 5]);
}
