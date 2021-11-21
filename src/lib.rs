#![deny(clippy::all)] // make all clippy warnings into errors
#![allow(clippy::many_single_char_names)]

mod canvas;
mod colors;
mod maths;
mod model;

pub use colors::*;

pub use canvas::Canvas;
pub use model::{Face, Model, ModelInput, Vertex};
