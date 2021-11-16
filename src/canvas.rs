use glam::IVec2;
use rgb::RGB8;

use crate::Model;

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

    /// Get a reference to the canvas's width.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Get a reference to the canvas's height.
    pub fn height(&self) -> usize {
        self.height
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

    #[inline]
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

    pub fn flip_y(&mut self) {
        let (width, height) = dbg!((self.width, self.height));

        for y in 0..height / 2 {
            let y0 = y * width;
            let y1 = (height - y - 1) * width;

            for x in 0..width {
                self.pixels.swap(y0 + x, y1 + x);
            }
        }
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
    pub fn line_fastest(
        &mut self,
        mut x0: i32,
        mut y0: i32,
        mut x1: i32,
        mut y1: i32,
        color: RGB8,
    ) {
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

    pub fn line(&mut self, p1: IVec2, p2: IVec2, color: RGB8) {
        let (x0, y0) = (p1.x, p1.y);
        let (x1, y1) = (p2.x, p2.y);
        self.line_fastest(x0, y0, x1, y1, color);
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

                self.line(IVec2::new(x0, y0), IVec2::new(x1, y1), color);
            }
        }
    }

    /// Output a wireframe (unfilled) triangle by using line drawing
    pub fn triangle_wireframe(&mut self, t0: IVec2, t1: IVec2, t2: IVec2, color: RGB8) {
        self.line(t0, t1, color);
        self.line(t1, t2, color);
        self.line(t2, t0, color);
    }

    /// Output a wireframe triangle with boundaries colored:
    /// * "Vertically longest" edge (from top vertex to bottom vertex) will be red
    /// * 2nd edge from bottom to middle vertex will be green
    /// * 3rd edge from middle to top vertex will be blue
    pub fn triangle_debug(&mut self, t0: IVec2, t1: IVec2, t2: IVec2) {
        let (t0, t1, t2) = {
            let mut vertices = [t0, t1, t2];
            vertices.sort_by(|a, b| a.y.cmp(&b.y));
            dbg!(vertices);
            (vertices[0], vertices[1], vertices[2])
        };

        self.line(t2, t0, RGB8::new(255, 0, 0));
        self.line(t0, t1, RGB8::new(0, 255, 0));
        self.line(t1, t2, RGB8::new(0, 0, 255));
    }

    // Draw a filled triangle using line sweeping.
    pub fn triangle_linesweep_orig(&mut self, t0: IVec2, t1: IVec2, t2: IVec2, color: RGB8) {
        // 1. sort the vertices by y coordinate, as prep for step 2
        let (t0, t1, t2) = {
            let mut vertices = [t0, t1, t2];
            vertices.sort_by(|a, b| a.y.cmp(&b.y));
            dbg!(vertices);
            (vertices[0], vertices[1], vertices[2])
        };

        // 2. Sweep from left to right. This is like outputting a "ladder" of strictly horizontal
        //    lines, with the rungs (lines) being attached to the left and right sides of the
        //    triangle, starting from the bottom vertex. However because it's a triangle, there will
        //    be a phase where the rungs get bigger first until the middle vertex is reached, then
        //    the rungs will get smaller again. So we split the sweeping (drawing of the ladder's
        //    rungs) up into 2 parts, starting with the bottom of the ladder:
        //   a) we start at the bottom most vertex (smallest y coordinate)
        //   b) we know that the top vertex (largest y coordinate) will be in a straight line with
        //      the bottom-most pixel
        //   c) that line will form one side of the ladder (side `a` - could be left or right
        //      depending on the triangle's orientation, aka "winding")
        //   d) then the "middle" vertex (by y coordinate) will be in between the other 2
        //   e) therefore we can interpolate from the bottom pixel to the middle pixel to find the
        //      other edge of the ladder.
        //   f) so then we draw a rung from one edge to the other and step up 1 y-pixel & repeat.
        let total_height = t2.y - t0.y;
        let segment_height = t1.y - t0.y + 1;
        for y in t0.y..=t1.y {
            // linearly interpolate position on the ladder's edges based on our current y-coordinate
            let alpha = (y - t0.y) as f32 / total_height as f32;
            let beta = (y - t0.y) as f32 / segment_height as f32;
            // a and b are points on the edges of the ladder
            let mut a = t0 + ((t2 - t0).as_vec2() * alpha).as_ivec2();
            let mut b = t0 + ((t1 - t0).as_vec2() * beta).as_ivec2();
            // we can only draw a line from left to right since we'll be incrementing the x
            // coordinate by 1 each time, so swap the vertices if necessary
            if a.x > b.x {
                std::mem::swap(&mut a, &mut b);
            }
            // 3. draw a horizontal line between the two endpoints
            for j in a.x..=b.x {
                *self.pixel(j, y) = color;
            }
        }

        // now repeat the same for the upper half of the triangle, from the middle vertex to the top
        // vertex.
        for y in t1.y..=t2.y {
            let segment_height = t2.y - t1.y + 1;
            let alpha = (y - t0.y) as f32 / total_height as f32;
            let beta = (y - t2.y) as f32 / segment_height as f32;
            let mut a = t0 + ((t2 - t0).as_vec2() * alpha).as_ivec2();
            // FIXED: the original code is wrong here, it was using t1 + diff instead of t2 + diff
            let mut b = t2 + ((t2 - t1).as_vec2() * beta).as_ivec2();
            if a.x > b.x {
                std::mem::swap(&mut a, &mut b);
            }
            for j in a.x..=b.x {
                *self.pixel(j, y) = color;
            }
        }
    }

    // Draw a filled triangle using line sweeping, approach 2
    pub fn triangle_linesweep_refined(&mut self, t0: IVec2, t1: IVec2, t2: IVec2, color: RGB8) {
        if t0.y == t1.y && t0.y == t2.y {
            return; // ignore degenerate triangles
        }

        let (t0, t1, t2) = {
            let mut vertices = [t0, t1, t2];
            vertices.sort_by(|a, b| a.y.cmp(&b.y));
            (vertices[0], vertices[1], vertices[2])
        };

        let total_height = t2.y - t0.y;
        for i in 0..total_height {
            let second_half = i > t1.y - t0.y || t1.y == t0.y;
            let segment_height = if second_half {
                t2.y - t1.y
            } else {
                t1.y - t0.y
            } as f32;

            let alpha = i as f32 / total_height as f32;
            let beta = (i - (if second_half { t1.y - t0.y } else { 0 })) as f32 / segment_height;

            let mut a = t0 + ((t2 - t0).as_vec2() * alpha).as_ivec2();
            let mut b = if second_half {
                t1 + ((t2 - t1).as_vec2() * beta).as_ivec2()
            } else {
                t0 + ((t1 - t0).as_vec2() * beta).as_ivec2()
            };

            if a.x > b.x {
                std::mem::swap(&mut a, &mut b);
            }
            for j in a.x..=b.x {
                *self.pixel(j, t0.y + i) = color;
            }
        }
    }
}
