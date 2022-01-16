use glam::{Mat3, Mat4, Vec2, Vec3, Vec4};

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
    fn vertex(&self, input: [Vertex; 3]) -> (Mat3, GouraudShaderState) {
        let mut varying_pos = Mat3::ZERO;
        let mut varying_uv = [Vec2::ZERO; 3];
        let mut varying_light_intensity = [0f32; 3];
        for (i, vert) in input.iter().enumerate() {
            *varying_pos.col_mut(i) = {
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
            tex.get_pixel(uv)
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

type VertexUVs = [Vec2; 3];

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
    /// normal texture must be in global coordinates
    normal_texture: &'t Texture,
}

impl<'t> NormalShader<'t> {
    pub fn new(
        viewport: Mat4,
        uniform_m: Mat4,
        light_dir: Vec3,
        diffuse_texture: &'t Texture,
        normal_texture_global: &'t Texture,
    ) -> NormalShader<'t> {
        Self {
            viewport,
            uniform_m,
            uniform_mit: uniform_m.inverse().transpose(),
            light_dir,
            diffuse_texture,
            normal_texture: normal_texture_global,
        }
    }
}

impl Shader<VertexUVs> for NormalShader<'_> {
    fn vertex(&self, input: [Vertex; 3]) -> (Mat3, VertexUVs) {
        let mut varying_pos = Mat3::ZERO;
        let mut varying_uv = [Vec2::ZERO; 3];
        for (i, vert) in input.iter().enumerate() {
            *varying_pos.col_mut(i) =
                (self.viewport * self.uniform_m).project_point3(vert.position);

            varying_uv[i] = Vec2::new(
                vert.uv.x * self.diffuse_texture.width as f32,
                vert.uv.y * self.diffuse_texture.height as f32,
            );
        }

        (varying_pos, varying_uv)
    }

    fn fragment(&self, barycentric_coords: Vec3, varying_uv: &VertexUVs) -> Option<RGB8> {
        let uv = varying_uv[0] * barycentric_coords[0]
            + varying_uv[1] * barycentric_coords[1]
            + varying_uv[2] * barycentric_coords[2];

        // correct normals for the affine transformation done in vertex shader
        let n = self
            .uniform_mit
            .project_point3(self.normal_texture.get_normal(uv))
            .normalize();
        let l = self.uniform_m.project_point3(self.light_dir).normalize();
        let intensity = crab_tv::yolo_max(0.0, n.dot(l));

        let unlit_color = self.diffuse_texture.get_pixel(uv);

        Some(unlit_color.map(|comp| (comp as f32 * intensity) as u8))
    }
}

/// A shader that handles normals correctly based on a global normal map
#[derive(Clone, Debug)]
pub struct PhongShader<'t> {
    viewport: Mat4,
    /// projection matrix * modelview matrix
    uniform_m: Mat4,
    /// projection matrix * modelview matrix then inverted & transposed, for correcting normals
    uniform_mit: Mat4,
    light_dir: Vec3,
    diffuse_texture: &'t Texture,
    /// normal texture must be in global coordinates
    normal_texture: &'t Texture,
    specular_texture: &'t Texture,
}

impl<'t> PhongShader<'t> {
    pub fn new(
        viewport: Mat4,
        uniform_m: Mat4,
        light_dir: Vec3,
        diffuse_texture: &'t Texture,
        normal_texture_global: &'t Texture,
        specular_texture: &'t Texture,
    ) -> PhongShader<'t> {
        Self {
            viewport,
            uniform_m,
            uniform_mit: uniform_m.inverse().transpose(),
            light_dir,
            diffuse_texture,
            normal_texture: normal_texture_global,
            specular_texture,
        }
    }
}

impl Shader<VertexUVs> for PhongShader<'_> {
    fn vertex(&self, input: [Vertex; 3]) -> (Mat3, VertexUVs) {
        let mut varying_tri = Mat3::ZERO;
        let mut varying_uv = [Vec2::ZERO; 3];
        for (i, vert) in input.iter().enumerate() {
            *varying_tri.col_mut(i) =
                (self.viewport * self.uniform_m).project_point3(vert.position);

            varying_uv[i] = Vec2::new(
                vert.uv.x * self.diffuse_texture.width as f32,
                vert.uv.y * self.diffuse_texture.height as f32,
            );
        }

        (varying_tri, varying_uv)
    }

    fn fragment(&self, barycentric_coords: Vec3, varying_uv: &VertexUVs) -> Option<RGB8> {
        let uv = varying_uv[0] * barycentric_coords[0]
            + varying_uv[1] * barycentric_coords[1]
            + varying_uv[2] * barycentric_coords[2];

        let n = self
            .uniform_mit
            .project_point3(self.normal_texture.get_normal(uv))
            .normalize();
        let l = self.uniform_m.project_point3(self.light_dir).normalize();
        let r = (n * (n.dot(l) * 2.0) - l).normalize(); // reflected light

        let unlit_color = self.diffuse_texture.get_pixel(uv);

        // calculate lighting intensity for this pixel
        let ambient_intensity = 1.0;
        let diffuse_intensity = crab_tv::yolo_max(0.0, n.dot(l));
        let specular_intensity =
            crab_tv::yolo_max(0.0, r.z).powf(self.specular_texture.get_specular(uv));

        // phong shading weights of each light component
        let ambient_weight = 5.0;
        let diffuse_weight = 1.0;
        let specular_weight = 0.6;

        Some(unlit_color.map(|comp| {
            (ambient_weight * ambient_intensity
                + comp as f32
                    * (diffuse_weight * diffuse_intensity + specular_weight * specular_intensity))
                as u8
        }))
    }
}

type DepthVaryingTri = Mat3;

/// Depth shader
#[derive(Clone, Debug)]
pub struct DepthShader {
    viewport: Mat4,
    /// projection matrix * modelview matrix
    uniform_m: Mat4,
}

impl DepthShader {
    pub fn new(viewport: Mat4, uniform_m: Mat4) -> DepthShader {
        Self {
            viewport,
            uniform_m,
        }
    }
}

impl Shader<DepthVaryingTri> for DepthShader {
    fn vertex(&self, input: [Vertex; 3]) -> (Mat3, DepthVaryingTri) {
        let mut varying_pos = Mat3::ZERO;
        let mut varying_tri = Mat3::ZERO;
        for (i, vert) in input.iter().enumerate() {
            *varying_pos.col_mut(i) =
                (self.viewport * self.uniform_m).project_point3(vert.position);

            *varying_tri.col_mut(i) = varying_pos.col(i);
        }

        (varying_pos, varying_tri)
    }

    fn fragment(&self, barycentric_coords: Vec3, varying_tri: &DepthVaryingTri) -> Option<RGB8> {
        let p = (*varying_tri) * barycentric_coords;
        let depth_scaled = p.z / crab_tv::DEPTH_MAX;
        Some(crab_tv::WHITE.map(|c| (c as f32 * depth_scaled) as u8))
    }
}
