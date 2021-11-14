use derive_more::{Add, AddAssign, Constructor, Display, Neg, Sub, SubAssign, Sum};

#[derive(
    Debug,
    Default,
    Copy,
    Clone,
    PartialEq,
    Constructor,
    Add,
    AddAssign,
    Sum,
    Sub,
    SubAssign,
    Display,
    Neg,
)]
#[display(fmt = "[{}, {}]", x, y)]
pub struct Vec2f {
    pub x: f32,
    pub y: f32,
}

#[derive(
    Debug,
    Default,
    Copy,
    Clone,
    PartialEq,
    Constructor,
    Add,
    AddAssign,
    Sum,
    Sub,
    SubAssign,
    Display,
    Neg,
)]
#[display(fmt = "[{}, {}]", x, y)]
pub struct Vec2i {
    pub x: i32,
    pub y: i32,
}
