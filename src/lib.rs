#![deny(clippy::all)] // make all clippy warnings into errors
#![allow(clippy::many_single_char_names)]

pub mod canvas;
pub mod model;

pub use canvas::Canvas;
pub use model::{Model, Vertex};
