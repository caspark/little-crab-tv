/// Legacy canvas API, where only certain fixed functions are supported (no shaders).
use glam::{IVec2, Mat4, Vec2, Vec3, Vec4};
use rgb::{ComponentMap, RGB8};

use crate::{
    maths::{self, yolo_max, yolo_min},
    model::Texture,
    Canvas, Model, DEPTH_MAX,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelShading {
    FlatOnly,
    DepthTested,
    Textured,
    Gouraud,
}

impl Canvas {
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

    pub fn model_wireframe(&mut self, model: &Model, color: RGB8) {
        for face in model.faces.iter() {
            for j in 0..3 {
                let v0 = model.vertices[face.points[j].vertices_index];
                let v1 = model.vertices[face.points[(j + 1) % 3].vertices_index];

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
                let x0 = ((v0.pos.x + 1.0) * (self.width() as f32 - 1.0) / 2.0) as i32;
                let y0 = ((v0.pos.y + 1.0) * (self.height() as f32 - 1.0) / 2.0) as i32;
                let x1 = ((v1.pos.x + 1.0) * (self.width() as f32 - 1.0) / 2.0) as i32;
                let y1 = ((v1.pos.y + 1.0) * (self.height() as f32 - 1.0) / 2.0) as i32;

                self.line(IVec2::new(x0, y0), IVec2::new(x1, y1), color);
            }
        }
    }

    pub fn model_colored_triangles(&mut self, model: &Model) {
        for face in model.faces.iter() {
            let mut screen_coords = [IVec2::new(0, 0); 3];
            for j in 0..3 {
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

                screen_coords[j] = IVec2::new(
                    ((v.pos.x + 1.0) * (self.width() as f32 - 1.0) / 2.0) as i32,
                    ((v.pos.y + 1.0) * (self.height() as f32 - 1.0) / 2.0) as i32,
                );
            }
            self.triangle_barycentric(&screen_coords, crate::colors::random_color());
        }
    }

    pub fn model_fixed_function(
        &mut self,
        model: &Model,
        light_dir: Vec3,
        shading: ModelShading,
        transform: Option<Mat4>,
    ) {
        // viewport matrix resizes/repositions the result to fit on screen
        fn viewport_transform(x: f32, y: f32, w: f32, h: f32) -> Mat4 {
            Mat4::from_cols(
                [w / 2.0, 0.0, 0.0, 0.0].into(),
                [0.0, h / 2.0, 0.0, 0.0].into(),
                [0.0, 0.0, DEPTH_MAX / 2.0, 0.0].into(),
                [x + w / 2.0, y + h / 2.0, DEPTH_MAX / 2.0, 1.0].into(),
            )
        }
        let viewport = viewport_transform(
            self.width() as f32 / 8.0,
            self.height() as f32 / 8.0,
            self.width() as f32 * 3.0 / 4.0,
            self.height() as f32 * 3.0 / 4.0,
        );

        let overall_transform = viewport * transform.unwrap_or(Mat4::IDENTITY);

        for face in model.faces.iter() {
            let mut screen_coords_2d = [IVec2::ZERO; 3];
            let mut screen_coords_3d = [Vec3::ZERO; 3];
            let mut world_coords = [Vec3::ZERO; 3];
            let mut texture_coords = [Vec2::ZERO; 3];
            for j in 0..3 {
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

                screen_coords_2d[j] = IVec2::new(
                    ((v.pos.x + 1.0) * (self.width() as f32 - 1.0) / 2.0) as i32,
                    ((v.pos.y + 1.0) * (self.height() as f32 - 1.0) / 2.0) as i32,
                );

                world_coords[j] = v.pos;

                // step 1 - embed into 4D space by converting to homogeneous coordinates
                let mut vec4: Vec4 = (v.pos, 1.0).into();
                // step 2 - multiply with projection & viewport matrices to correct perspective
                vec4 = overall_transform * vec4;
                // step 3 - divide by w to reproject into 3d screen coordinates
                screen_coords_3d[j] = Vec3::new(vec4.x / vec4.w, vec4.y / vec4.w, vec4.z / vec4.w);

                let raw_texture_coords = model.texture_coords[face.points[j].uv_index];
                texture_coords[j] = Vec2::new(
                    raw_texture_coords.x * model.diffuse_texture.width as f32,
                    raw_texture_coords.y * model.diffuse_texture.height as f32,
                );
            }

            let mut vertex_intensity = [0.0f32; 3];
            if shading == ModelShading::Gouraud {
                for j in 0..3 {
                    vertex_intensity[j] =
                        model.vertex_normals[face.points[j].normals_index].dot(light_dir);
                }
            } else {
                let n =
                    (world_coords[2] - world_coords[0]).cross(world_coords[1] - world_coords[0]);
                let n = n.normalize();
                let intensity: f32 = n.dot(-light_dir);
                for j in 0..3 {
                    vertex_intensity[j] = intensity;
                }
            };

            if vertex_intensity.iter().any(|i| *i > 0.0) {
                let avg_intensity =
                    vertex_intensity.iter().sum::<f32>() / vertex_intensity.len() as f32;
                let w = (avg_intensity * 255.0) as u8;
                match shading {
                    ModelShading::FlatOnly => {
                        self.triangle_barycentric(&screen_coords_2d, RGB8::new(w, w, w))
                    }
                    ModelShading::DepthTested => self
                        .triangle_barycentric_depth_tested(&screen_coords_3d, RGB8::new(w, w, w)),
                    ModelShading::Textured => self.triangle_barycentric_texture(
                        &screen_coords_3d,
                        &model.diffuse_texture,
                        &texture_coords,
                        avg_intensity,
                    ),
                    ModelShading::Gouraud => self.triangle_barycentric_gouraud(
                        &screen_coords_3d,
                        &model.diffuse_texture,
                        &texture_coords,
                        &vertex_intensity,
                    ),
                }
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
            (vertices[0], vertices[1], vertices[2])
        };

        self.line(t2, t0, RGB8::new(255, 0, 0));
        self.line(t0, t1, RGB8::new(0, 255, 0));
        self.line(t1, t2, RGB8::new(0, 0, 255));
    }

    // Draw a filled triangle using line sweeping.
    pub fn triangle_linesweep_verbose(&mut self, pts: &[IVec2], color: RGB8) {
        let (t0, t1, t2) = (pts[0], pts[1], pts[2]);

        if t0.y == t1.y && t0.y == t2.y {
            return; // ignore degenerate triangles
        }

        // 1. sort the vertices by y coordinate, as prep for step 2
        let (t0, t1, t2) = {
            let mut vertices = [t0, t1, t2];
            vertices.sort_by(|a, b| a.y.cmp(&b.y));
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
        for y in t0.y..=t1.y {
            // FIXED: the original code was adding 1 to the y coordinate, which causes wonky triangles.
            let segment_height = t1.y - t0.y;
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
            // FIXED: the original code was adding 1 to the y coordinate, which causes wonky triangles.
            let segment_height = t2.y - t1.y;
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
    pub fn triangle_linesweep_compact(&mut self, pts: &[IVec2], color: RGB8) {
        let (t0, t1, t2) = (pts[0], pts[1], pts[2]);

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

    pub fn triangle_barycentric(&mut self, pts: &[IVec2], color: RGB8) {
        let mut bboxmin = IVec2::new((self.width() - 1) as i32, (self.height() - 1) as i32);
        let mut bboxmax = IVec2::new(0, 0);
        let clamp = IVec2::new((self.width() - 1) as i32, (self.height() - 1) as i32);

        for i in 0..3 {
            for j in 0..2 {
                bboxmin[j] = std::cmp::max(0, std::cmp::min(bboxmin[j], pts[i][j]));
                bboxmax[j] = std::cmp::min(clamp[j], std::cmp::max(bboxmax[j], pts[i][j]));
            }
        }

        let mut p: IVec2;
        for i in bboxmin.x..=bboxmax.x {
            for j in bboxmin.y..=bboxmax.y {
                p = IVec2::new(i, j);
                let bc_screen = maths::barycentric_coords_2d(pts, p);
                if bc_screen.x >= 0.0 && bc_screen.y >= 0.0 && bc_screen.z >= 0.0 {
                    *self.pixel(i, j) = color;
                }
            }
        }
    }

    pub fn triangle_barycentric_depth_tested(&mut self, pts: &[Vec3], color: RGB8) {
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
                let bc_screen = maths::barycentric_coords_3d(pts, p);
                if bc_screen.x < 0.0 || bc_screen.y < 0.0 || bc_screen.z < 0.0 {
                    continue;
                }
                let mut pixel_z = 0.0;
                for k in 0..3 {
                    pixel_z += pts[k][2] * bc_screen[k];
                }
                let z_buf_for_pixel = self.z_buffer_at(i, j);
                if *z_buf_for_pixel < pixel_z {
                    *z_buf_for_pixel = pixel_z;
                    *self.pixel(i, j) = color;
                }
            }
        }
    }

    pub fn triangle_barycentric_texture(
        &mut self,
        pts: &[Vec3],
        tex: &Texture,
        varying_uv: &[Vec2],
        light_intensity: f32,
    ) {
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
                let bc_screen = maths::barycentric_coords_3d(pts, p);
                if bc_screen.x < 0.0 || bc_screen.y < 0.0 || bc_screen.z < 0.0 {
                    continue;
                }
                let mut pixel_z = 0.0;
                for k in 0..3 {
                    pixel_z += pts[k][2] * bc_screen[k];
                }
                let z_buf_for_pixel = self.z_buffer_at(i, j);
                if *z_buf_for_pixel < pixel_z {
                    *z_buf_for_pixel = pixel_z;

                    let uv = varying_uv[0] * bc_screen[0]
                        + varying_uv[1] * bc_screen[1]
                        + varying_uv[2] * bc_screen[2];

                    let color = tex.data[(tex.height - uv.y as usize) * tex.width + uv.x as usize]
                        .map(|comp| (comp as f32 * light_intensity) as u8);

                    *self.pixel(i, j) = color;
                }
            }
        }
    }

    pub fn triangle_barycentric_gouraud(
        &mut self,
        pts: &[Vec3],
        tex: &Texture,
        varying_uv: &[Vec2],
        light_intensity: &[f32],
    ) {
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
                let bc_screen = maths::barycentric_coords_3d(pts, p);
                if bc_screen.x < 0.0 || bc_screen.y < 0.0 || bc_screen.z < 0.0 {
                    continue;
                }
                let mut pixel_z = 0.0;
                for k in 0..3 {
                    pixel_z += pts[k][2] * bc_screen[k];
                }
                let z_buf_for_pixel = self.z_buffer_at(i, j);
                if *z_buf_for_pixel < pixel_z {
                    *z_buf_for_pixel = pixel_z;

                    let uv = varying_uv[0] * bc_screen[0]
                        + varying_uv[1] * bc_screen[1]
                        + varying_uv[2] * bc_screen[2];
                    // the bit that differs from standard flat shading: interpolate the light
                    // intensity using barycentric coordinates of this pixel
                    let weighted_light_intensity = light_intensity[0] * bc_screen[0]
                        + light_intensity[1] * bc_screen[1]
                        + light_intensity[2] * bc_screen[2];

                    let color = tex.data[(tex.height - uv.y as usize) * tex.width + uv.x as usize]
                        .map(|comp| (comp as f32 * weighted_light_intensity) as u8);

                    *self.pixel(i, j) = color;
                }
            }
        }
    }
}
