#![deny(clippy::all)] // make all clippy warnings into errors
#![allow(clippy::many_single_char_names)]

mod canvas;
mod canvas_legacy;
mod colors;
mod maths;
mod model;

pub use colors::*;

pub use canvas::{Canvas, Shader, VertexShaderInput, VertexShaderOutput};
pub use canvas_legacy::ModelShading;
pub use model::{Face, Model, ModelInput, Texture, Vertex};
