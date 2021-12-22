use glam::{Vec2, Vec3};
use rgb::RGB8;

use crate::{
    maths::{self, yolo_max, yolo_min},
    Model,
};

#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct Vertex {
    pub position: Vec3,
    pub uv: Vec2,
    pub normal: Vec3,
}

pub trait Shader {
    fn vertex(&mut self, triangle: [Vertex; 3]) -> [Vec3; 3];
    fn fragment(&self, barycentric_coords: Vec3) -> Option<RGB8>;
}

#[derive(Clone, Debug)]
pub struct Canvas {
    width: usize,
    height: usize,
    pixels: Vec<RGB8>,
    z_buffer: Vec<f32>,
}

impl Canvas {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![RGB8::default(); width * height],
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

    #[inline]
    pub fn z_buffer_at(&mut self, x: i32, y: i32) -> &mut f32 {
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

    pub fn model_shader(&mut self, model: &Model, shader: &mut dyn Shader) {
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

            let screen_coords = shader.vertex(vertices);

            self.triangle_shader(screen_coords, shader);
        }
    }

    pub fn triangle_shader(&mut self, pts: [Vec3; 3], shader: &mut dyn Shader) {
        let mut bboxmin = Vec2::new((self.width() - 1) as f32, (self.height() - 1) as f32);
        let mut bboxmax = Vec2::new(0.0, 0.0);
        let clamp = Vec2::new((self.width() - 1) as f32, (self.height() - 1) as f32);

        for i in 0..3 {
            for j in 0..2 {
                bboxmin[j] = yolo_max(0.0, yolo_min(bboxmin[j], pts[i][j]));
                bboxmax[j] = yolo_min(clamp[j], yolo_max(bboxmax[j], pts[i][j]));
            }
        }

        for i in (bboxmin.x as i32)..=(bboxmax.x as i32) {
            for j in (bboxmin.y as i32)..=(bboxmax.y as i32) {
                let p = Vec2::new(i as f32, j as f32);
                let bc_screen = maths::barycentric_coords_3d(&pts, p);
                if bc_screen.x < 0.0 || bc_screen.y < 0.0 || bc_screen.z < 0.0 {
                    continue;
                }
                let mut pixel_z = 0.0;
                for k in 0..3 {
                    pixel_z += pts[k][2] * bc_screen[k];
                }
                let z_buf_for_pixel = self.z_buffer_at(i, j);
                if *z_buf_for_pixel < pixel_z {
                    let maybe_color = shader.fragment(bc_screen);
                    if let Some(color) = maybe_color {
                        *z_buf_for_pixel = pixel_z;
                        *self.pixel(i, j) = color;
                    }
                }
            }
        }
    }
}
