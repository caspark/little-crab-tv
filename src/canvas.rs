use std::f32::consts::PI;

use glam::{Mat3, Vec2, Vec3};
use rgb::{ComponentMap, RGBA8};

use crate::{
    maths::{self, yolo_max, yolo_min},
    Model, CLEAR, DEPTH_MAX,
};

#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct Vertex {
    pub position: Vec3,
    pub uv: Vec2,
    pub normal: Vec3,
}

pub trait Shader<S> {
    fn vertex(&self, triangle: [Vertex; 3]) -> (Mat3, S);
    fn fragment(&self, barycentric_coords: Vec3, state: &S) -> Option<RGBA8>;
}

#[derive(Clone, Debug)]
pub struct Canvas {
    width: usize,
    height: usize,
    pixels: Vec<RGBA8>,
    z_buffer: Vec<f32>,
}

impl Canvas {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![RGBA8::default(); width * height],
            z_buffer: vec![f32::NEG_INFINITY; width * height],
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

    pub fn pixels(&self) -> &[RGBA8] {
        &self.pixels
    }

    pub fn pixels_mut(&mut self) -> &mut [RGBA8] {
        &mut self.pixels
    }

    #[inline]
    pub fn pixel(&self, x: i32, y: i32) -> RGBA8 {
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
        self.pixels[y as usize * self.width + x as usize]
    }

    #[inline]
    pub fn pixel_mut(&mut self, x: i32, y: i32) -> &mut RGBA8 {
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

    #[inline]
    pub fn z_buffer_at(&self, x: i32, y: i32) -> f32 {
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
        self.z_buffer[y as usize * self.width + x as usize]
    }

    #[inline]
    pub fn z_buffer_at_mut(&mut self, x: i32, y: i32) -> &mut f32 {
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
        &mut self.z_buffer[y as usize * self.width + x as usize]
    }

    pub fn replace_with_z_buffer(&mut self) {
        self.pixels = self
            .z_buffer
            .iter()
            .map(|d| (*d * 255.0 / DEPTH_MAX) as u8)
            .map(|c| RGBA8::new(c, c, c, 255))
            .collect();
    }

    pub fn flip_y(&mut self) {
        let (width, height) = (self.width, self.height);

        for y in 0..height / 2 {
            let y0 = y * width;
            let y1 = (height - y - 1) * width;

            for x in 0..width {
                self.pixels.swap(y0 + x, y1 + x);
            }
        }
    }

    pub fn model_shader<S>(&mut self, model: &Model, shader: &dyn Shader<S>) {
        for face in model.faces.iter() {
            let mut vertices = [Vertex::default(); 3];
            for j in 0..3 {
                vertices[j] = Vertex {
                    position: {
                        let v = model.vertices[face.points[j].vertices_index];
                        // this simplistic rendering code assumes that the vertice coordinates are
                        // between -1 and 1, so confirm that assumption
                        debug_assert!(
                            -1.0 <= v.pos.x && v.pos.x <= 1.0,
                            "x coordinate out of range: {}",
                            v.pos.x
                        );
                        debug_assert!(
                            -1.0 <= v.pos.y && v.pos.y <= 1.0,
                            "y coordinate out of range: {}",
                            v.pos.y
                        );
                        v.pos
                    },
                    uv: model.texture_coords[face.points[j].uv_index],
                    normal: model.vertex_normals[face.points[j].normals_index],
                }
            }

            let (screen_coords, shader_state) = shader.vertex(vertices);

            self.triangle_shader(screen_coords, shader, shader_state);
        }
    }

    pub fn triangle_shader<S>(&mut self, pts: Mat3, shader: &dyn Shader<S>, shader_state: S) {
        let mut bboxmin = Vec2::new((self.width() - 1) as f32, (self.height() - 1) as f32);
        let mut bboxmax = Vec2::new(0.0, 0.0);
        let clamp = Vec2::new((self.width() - 1) as f32, (self.height() - 1) as f32);

        for i in 0..3 {
            for j in 0..2 {
                bboxmin[j] = yolo_max(0.0, yolo_min(bboxmin[j], pts.col(i)[j]));
                bboxmax[j] = yolo_min(clamp[j], yolo_max(bboxmax[j], pts.col(i)[j]));
            }
        }

        for i in (bboxmin.x as i32)..=(bboxmax.x as i32) {
            for j in (bboxmin.y as i32)..=(bboxmax.y as i32) {
                let p = Vec2::new(i as f32, j as f32);
                let bc_screen = maths::barycentric_coords_3d_matrix(pts, p);
                if bc_screen.x < 0.0 || bc_screen.y < 0.0 || bc_screen.z < 0.0 {
                    continue;
                }
                let mut pixel_z = 0.0;
                for k in 0..3 {
                    pixel_z += pts.col(k)[2] * bc_screen[k];
                }
                let z_buf_for_pixel = self.z_buffer_at_mut(i, j);
                if *z_buf_for_pixel < pixel_z {
                    let maybe_color = shader.fragment(bc_screen, &shader_state);
                    if let Some(color) = maybe_color {
                        *z_buf_for_pixel = pixel_z;
                        *self.pixel_mut(i, j) = color;
                    }
                }
            }
        }
    }

    pub fn apply_ambient_occlusion(&mut self, strength: f32, ambient_occlusion_passes: usize) {
        for x in 0..self.width() {
            for y in 0..self.height() {
                if (*self.z_buffer_at_mut(x as i32, y as i32)) < -1e5 {
                    continue;
                }

                let mut total = 0.0;
                let mut a = 0.0;
                while a < PI * 2.0 - 1e-4 {
                    total += PI / 2.0
                        - max_elevation_angle(
                            self,
                            Vec2::new(x as f32, y as f32),
                            Vec2::new(a.cos(), a.sin()),
                            ambient_occlusion_passes,
                        );
                    a += PI / 4.0;
                }

                total /= PI / 2.0 * 8.0;
                total = total.powf(strength);
                *self.pixel_mut(x as i32, y as i32) = self
                    .pixel(x as i32, y as i32)
                    .map(|c| (total * c as f32) as u8);
            }
        }
    }
}

fn max_elevation_angle(image: &Canvas, p: Vec2, dir: Vec2, samples: usize) -> f32 {
    let mut max_angle = 0.0;

    let mut t = 0.0;
    while t < samples as f32 {
        let cur = p + dir * t;
        t += 1.0;

        if cur.x >= image.width() as f32
            || cur.y >= image.height() as f32
            || cur.x < 0.0
            || cur.y < 0.0
        {
            return max_angle;
        }

        let distance = (p - cur).length();
        if distance < 1.0 {
            continue;
        }

        let elevation = image.z_buffer_at(cur.x as i32, cur.y as i32)
            - image.z_buffer_at(p.x as i32, p.y as i32);
        max_angle = max_angle.max((elevation / distance).atan());
    }
    max_angle
}
