//! module for pixels ops.
#![allow(unused_imports)]
pub mod blending;
mod utility;
mod wam;
pub use blending::Blend;
pub(crate) use utility::{Floatify, Unfloatify, float, unfloat};
pub(crate) use wam::Wam;
pub mod convert;
