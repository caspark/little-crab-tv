use glam::{Mat4, Vec2, Vec3, Vec4};

use crab_tv::{Shader, Texture, Vertex};
use rgb::{ComponentMap, RGB8};

#[derive(Clone, Debug)]

pub struct GouraudShaderState {
    varying_uv: [Vec2; 3],
    varying_light_intensity: [f32; 3],
}

#[derive(Clone, Debug)]
pub struct GouraudShader<'t> {
    vertex_transform: Mat4,
    light_dir: Vec3,
    diffuse_texture: Option<&'t Texture>,
    bucket_light_intensity: bool,
}

impl<'t> GouraudShader<'t> {
    pub fn new(
        viewport: Mat4,
        uniform_m: Mat4, // projection matrix * modelview matrix
        light_dir: Vec3,
        diffuse_texture: Option<&'t Texture>,
        bucket_light_intensity: bool,
    ) -> GouraudShader<'t> {
        Self {
            vertex_transform: viewport * uniform_m,
            light_dir,
            diffuse_texture,
            bucket_light_intensity,
        }
    }
}

impl Shader<GouraudShaderState> for GouraudShader<'_> {
    fn vertex(&self, input: [Vertex; 3]) -> ([Vec3; 3], GouraudShaderState) {
        let mut varying_pos = [Vec3::ZERO; 3];
        let mut varying_uv = [Vec2::ZERO; 3];
        let mut varying_light_intensity = [0f32; 3];
        for (i, vert) in input.iter().enumerate() {
            varying_pos[i] = {
                // Transform the vertex position
                // step 1 - embed into 4D space by converting to homogeneous coordinates
                let mut vec4: Vec4 = (vert.position, 1.0).into();
                // step 2 - multiply with projection & viewport matrices to correct perspective
                vec4 = self.vertex_transform * vec4;
                // step 3 - divide by w to reproject into 3d screen coordinates
                Vec3::new(vec4.x / vec4.w, vec4.y / vec4.w, vec4.z / vec4.w)
            };

            // Transform the vertex texture coordinates based on the texture we have
            varying_uv[i] = if let Some(ref texture) = self.diffuse_texture {
                Vec2::new(
                    vert.uv.x * texture.width as f32,
                    vert.uv.y * texture.height as f32,
                )
            } else {
                vert.uv
            };

            // Calculate the light intensity
            varying_light_intensity[i] = vert.normal.dot(self.light_dir);
        }

        (
            varying_pos,
            GouraudShaderState {
                varying_uv,
                varying_light_intensity,
            },
        )
    }

    fn fragment(&self, barycentric_coords: Vec3, state: &GouraudShaderState) -> Option<RGB8> {
        let GouraudShaderState {
            varying_uv,
            varying_light_intensity: light_intensity,
        } = state;

        let uv = varying_uv[0] * barycentric_coords[0]
            + varying_uv[1] * barycentric_coords[1]
            + varying_uv[2] * barycentric_coords[2];

        let weighted_light_intensity = light_intensity[0] * barycentric_coords[0]
            + light_intensity[1] * barycentric_coords[1]
            + light_intensity[2] * barycentric_coords[2];

        let weighted_light_intensity = if self.bucket_light_intensity {
            bucket_intensity(weighted_light_intensity)
        } else {
            weighted_light_intensity
        };

        let unlit_color = if let Some(ref tex) = self.diffuse_texture {
            tex.get_pixel(uv.x as usize, uv.y as usize)
        } else {
            crab_tv::WHITE
        };

        Some(unlit_color.map(|comp| (comp as f32 * weighted_light_intensity) as u8))
    }
}

fn bucket_intensity(intensity: f32) -> f32 {
    if intensity > 0.85 {
        1.0
    } else if intensity > 0.60 {
        0.80
    } else if intensity > 0.45 {
        0.60
    } else if intensity > 0.30 {
        0.45
    } else if intensity > 0.15 {
        0.30
    } else {
        0.0
    }
}

type NormalShaderState = [Vec2; 3];

/// A shader that handles normals correctly based on a global normal map
#[derive(Clone, Debug)]
pub struct NormalShader<'t> {
    viewport: Mat4,
    /// projection matrix * modelview matrix
    uniform_m: Mat4,
    /// projection matrix * modelview matrix then inverted & transposed, for correcting normals
    uniform_mit: Mat4,
    light_dir: Vec3,
    diffuse_texture: &'t Texture,
    normal_texture: &'t Texture,
}

impl<'t> NormalShader<'t> {
    pub fn new(
        viewport: Mat4,
        uniform_m: Mat4,
        light_dir: Vec3,
        diffuse_texture: &'t Texture,
        normal_texture: &'t Texture,
    ) -> NormalShader<'t> {
        Self {
            viewport,
            uniform_m,
            uniform_mit: uniform_m.inverse().transpose(),
            light_dir,
            diffuse_texture,
            normal_texture,
        }
    }
}

impl Shader<NormalShaderState> for NormalShader<'_> {
    fn vertex(&self, input: [Vertex; 3]) -> ([Vec3; 3], NormalShaderState) {
        let mut varying_pos = [Vec3::ZERO; 3];
        let mut varying_uv = [Vec2::ZERO; 3];
        for (i, vert) in input.iter().enumerate() {
            varying_pos[i] = (self.viewport * self.uniform_m).project_point3(vert.position);

            // Transform the vertex texture coordinates based on the texture we have
            varying_uv[i] = Vec2::new(
                vert.uv.x * self.diffuse_texture.width as f32,
                vert.uv.y * self.diffuse_texture.height as f32,
            );
        }

        (varying_pos, varying_uv)
    }

    fn fragment(&self, barycentric_coords: Vec3, varying_uv: &NormalShaderState) -> Option<RGB8> {
        let uv = varying_uv[0] * barycentric_coords[0]
            + varying_uv[1] * barycentric_coords[1]
            + varying_uv[2] * barycentric_coords[2];

        let n = {
            let pixel = self
                .normal_texture
                .get_pixel(uv.x as usize, uv.y as usize)
                // now normalize to [-1.0, 1.0]
                .map(|comp| comp as f32 / 255.0 * 2.0 - 1.0);
            // correct normals for the affine transformation done in vertex shader
            self.uniform_mit
                .project_point3(Vec3::new(pixel.r, pixel.g, pixel.b))
                .normalize()
        };
        let l = self.uniform_m.project_point3(self.light_dir).normalize();
        let weighted_light_intensity = crab_tv::yolo_max(0.0, n.dot(l));

        let unlit_color = self.diffuse_texture.get_pixel(uv.x as usize, uv.y as usize);

        Some(unlit_color.map(|comp| (comp as f32 * weighted_light_intensity) as u8))
    }
}
