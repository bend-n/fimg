//! module for pixels ops.
#![allow(unused_imports)]
pub mod blending;
mod utility;
mod wam;
pub use blending::Blend;
pub(crate) use utility::{float, unfloat, Floatify, PMap, Trunc, Unfloatify};
pub(crate) use wam::Wam;
