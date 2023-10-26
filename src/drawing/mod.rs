//! contains drawing operations, like
//! {line, box, triangle, polygon, circle, text}
//! drawing
mod r#box;
mod circle;
mod line;
mod poly;
#[cfg(feature = "text")]
mod text;
mod tri;
