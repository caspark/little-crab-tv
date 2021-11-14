use rgb::RGB8;

use crate::{maths::Vec2i, Model};

#[derive(Clone, Debug)]
pub struct Canvas {
    width: usize,
    height: usize,
    pixels: Vec<RGB8>,
}

impl Canvas {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![RGB8::default(); width * height],
        }
    }

    pub fn pixels(&self) -> &[RGB8] {
        &self.pixels
    }

    pub fn pixels_mut(&mut self) -> &mut [RGB8] {
        &mut self.pixels
    }

    pub fn into_pixels(self) -> Vec<RGB8> {
        self.pixels
    }

    pub fn pixel(&mut self, x: i32, y: i32) -> &mut RGB8 {
        debug_assert!(
            x >= 0 && x < self.width as i32,
            "x coordinate of '{}' is out of bounds 0 to {}",
            x,
            self.width as i32
        );
        debug_assert!(
            y >= 0 && y < self.height as i32,
            "y coordinate of '{}' is out of bounds 0 to {}",
            y,
            self.height as i32
        );
        &mut self.pixels[y as usize * self.width + x as usize]
    }

    // incorrect because it depends on choosing the correct "increment", which will vary based on
    // how many pixels need to be drawn
    pub fn line_naive1(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: RGB8) {
        let increment = 0.1;
        for i in 0..((1.0 / increment) as i32) {
            let i = f64::from(i) * increment;
            let x = x0 as f64 + (x1 - x0) as f64 * i;
            let y = y0 as f64 + (y1 - y0) as f64 * i;
            *self.pixel(x as i32, y as i32) = color;
        }
    }

    // incorrect because it doesn't handle the case where the line is near vertical or x1 < x0
    pub fn line_naive2(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: RGB8) {
        for x in x0..x1 {
            let t = (x - x0) as f64 / (x1 - x0) as f64;
            let y = y0 as f64 * (1.0 - t) as f64 + y1 as f64 * t as f64;
            *self.pixel(x as i32, y as i32) = color;
        }
    }

    // Bresenham's algorithm 1 - correct but slow due to needing floating point maths
    pub fn line_slow(&mut self, mut x0: i32, mut y0: i32, mut x1: i32, mut y1: i32, color: RGB8) {
        let steep = if (x0 - x1).abs() < (y0 - y1).abs() {
            std::mem::swap(&mut x0, &mut y0);
            std::mem::swap(&mut x1, &mut y1);
            true
        } else {
            false
        };

        if x0 > x1 {
            std::mem::swap(&mut x0, &mut x1);
            std::mem::swap(&mut y0, &mut y1);
        }

        let divisor = x1 - x0;
        for x in x0..x1 {
            let t = (x - x0) as f64 / divisor as f64;
            let y = y0 as f64 * (1.0 - t) as f64 + y1 as f64 * t as f64;
            if steep {
                *self.pixel(y as i32, x as i32) = color;
            } else {
                *self.pixel(x as i32, y as i32) = color;
            }
        }
    }

    // Bresenham's algorithm 2 - still using floating point maths but avoiding some division
    pub fn line_faster(&mut self, mut x0: i32, mut y0: i32, mut x1: i32, mut y1: i32, color: RGB8) {
        let steep = if (x0 - x1).abs() < (y0 - y1).abs() {
            std::mem::swap(&mut x0, &mut y0);
            std::mem::swap(&mut x1, &mut y1);
            true
        } else {
            false
        };

        if x0 > x1 {
            std::mem::swap(&mut x0, &mut x1);
            std::mem::swap(&mut y0, &mut y1);
        }

        let dx = x1 - x0;
        let dy = y1 - y0;
        let derror = (dy as f64 / dx as f64).abs();
        let mut error = 0.0;
        let mut y = y0;
        for x in x0..x1 {
            if steep {
                *self.pixel(y, x) = color;
            } else {
                *self.pixel(x, y) = color;
            }
            error += derror;
            if error > 0.5 {
                y += if y1 > y0 { 1 } else { -1 };
                error -= 1.0;
            }
        }
    }

    // Bresenham's algorithm 3 - correct & fastest, using integer maths instead of floating point
    pub fn line(&mut self, mut x0: i32, mut y0: i32, mut x1: i32, mut y1: i32, color: RGB8) {
        let steep = if (x0 - x1).abs() < (y0 - y1).abs() {
            std::mem::swap(&mut x0, &mut y0);
            std::mem::swap(&mut x1, &mut y1);
            true
        } else {
            false
        };

        if x0 > x1 {
            std::mem::swap(&mut x0, &mut x1);
            std::mem::swap(&mut y0, &mut y1);
        }

        let dx = x1 - x0;
        let dy = y1 - y0;
        let derror2 = dy.abs() * 2;
        let mut error2 = 0;
        let mut y = y0;
        for x in x0..x1 {
            if steep {
                *self.pixel(y as i32, x as i32) = color;
            } else {
                *self.pixel(x as i32, y as i32) = color;
            }
            error2 += derror2;
            if error2 > dx {
                y += if y1 > y0 { 1 } else { -1 };
                error2 -= dx * 2;
            }
        }
    }

    pub fn wireframe(&mut self, model: &Model, color: RGB8) {
        for face in model.faces.iter() {
            for j in 0..3 {
                let v0 = model.vertices[face.vertices[j]];
                debug_assert!(
                    face.vertices.len() == 3,
                    "only faces with exactly 3 vertices are supported; found {} vertices",
                    face.vertices.len()
                );

                let v1 = model.vertices[face.vertices[(j + 1) % 3]];

                // this simplistic rendering code assumes that the vertice coordinates are
                // between -1 and 1, so confirm that assumption
                debug_assert!(
                    -1.0 <= v0.pos.x && v0.pos.x <= 1.0,
                    "x coordinate out of range: {}",
                    v0.pos.x
                );
                debug_assert!(
                    -1.0 <= v0.pos.y && v0.pos.y <= 1.0,
                    "y coordinate out of range: {}",
                    v0.pos.y
                );
                debug_assert!(
                    -1.0 <= v1.pos.x && v1.pos.x <= 1.0,
                    "x coordinate out of range: {}",
                    v1.pos.x
                );
                debug_assert!(
                    -1.0 <= v1.pos.y && v1.pos.y <= 1.0,
                    "y coordinate out of range: {}",
                    v1.pos.y
                );
                let x0 = ((v0.pos.x + 1.0) * (self.width as f32 - 1.0) / 2.0) as i32;
                let y0 = ((v0.pos.y + 1.0) * (self.height as f32 - 1.0) / 2.0) as i32;
                let x1 = ((v1.pos.x + 1.0) * (self.width as f32 - 1.0) / 2.0) as i32;
                let y1 = ((v1.pos.y + 1.0) * (self.height as f32 - 1.0) / 2.0) as i32;

                self.line(x0, y0, x1, y1, color);
            }
        }
    }

    pub fn flip_y(&mut self) {
        self.pixels.reverse();
    }

    pub fn triangle(&mut self, t0: Vec2i, t1: Vec2i, t2: Vec2i, color: RGB8) {
        self.line(t0.x, t0.y, t1.x, t1.y, color);
        self.line(t1.x, t1.y, t2.x, t2.y, color);
        self.line(t2.x, t2.y, t0.x, t0.y, color);
    }
}
