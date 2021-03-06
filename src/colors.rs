use rgb::RGB8;

pub const WHITE: RGB8 = RGB8::new(255, 255, 255);
pub const BLACK: RGB8 = RGB8::new(0, 0, 0);

pub const RED: RGB8 = RGB8::new(255, 0, 0);
pub const GREEN: RGB8 = RGB8::new(0, 255, 0);
pub const BLUE: RGB8 = RGB8::new(0, 0, 255);

pub const YELLOW: RGB8 = RGB8::new(255, 255, 0);
pub const CYAN: RGB8 = RGB8::new(0, 255, 255);
pub const MAGENTA: RGB8 = RGB8::new(255, 0, 255);

pub fn random_color() -> RGB8 {
    RGB8::new(
        rand::random::<u8>() % 255,
        rand::random::<u8>() % 255,
        rand::random::<u8>() % 255,
    )
}
