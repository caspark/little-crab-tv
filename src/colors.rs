use rgb::RGBA8;

pub const WHITE: RGBA8 = RGBA8::new(255, 255, 255, 255);
pub const BLACK: RGBA8 = RGBA8::new(0, 0, 0, 255);
pub const CLEAR: RGBA8 = RGBA8::new(100, 100, 100, 0);

pub const RED: RGBA8 = RGBA8::new(255, 0, 0, 255);
pub const GREEN: RGBA8 = RGBA8::new(0, 255, 0, 255);
pub const BLUE: RGBA8 = RGBA8::new(0, 0, 255, 255);

pub const YELLOW: RGBA8 = RGBA8::new(255, 255, 0, 255);
pub const CYAN: RGBA8 = RGBA8::new(0, 255, 255, 255);
pub const MAGENTA: RGBA8 = RGBA8::new(255, 0, 255, 255);

pub fn random_color() -> RGBA8 {
    RGBA8::new(
        rand::random::<u8>() % 255,
        rand::random::<u8>() % 255,
        rand::random::<u8>() % 255,
        255,
    )
}
