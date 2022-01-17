#![deny(clippy::all)] // make all clippy warnings into errors
#![allow(clippy::many_single_char_names)]
#![allow(clippy::needless_range_loop)]

mod canvas;
mod canvas_legacy;
mod colors;
mod maths;
mod model;

pub use colors::*;

pub use canvas::{Canvas, Shader, Vertex};
pub use canvas_legacy::ModelShading;
pub use maths::{look_at_transform, viewport_transform, yolo_max, yolo_min, DEPTH_MAX};
pub use model::{Face, Model, ModelInput, Texture};
