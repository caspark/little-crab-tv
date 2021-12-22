use glam::{Mat4, Vec2, Vec3, Vec4};

use crab_tv::{Shader, Texture, Vertex};
use rgb::{ComponentMap, RGB8};

#[derive(Clone, Debug)]

pub struct GouraudShaderState {
    varying_uv: [Vec2; 3],
    light_intensity: [f32; 3],
}

#[derive(Clone, Debug)]
pub struct GouraudShader<'t> {
    light_dir: Vec3,
    diffuse_texture: Option<&'t Texture>,
    vertex_transform: Mat4,
}

impl<'t> GouraudShader<'t> {
    pub fn new(
        vertex_transform: Mat4,
        light_dir: Vec3,
        diffuse_texture: Option<&'t Texture>,
    ) -> GouraudShader {
        Self {
            vertex_transform,
            light_dir,
            diffuse_texture,
        }
    }
}

impl Shader<GouraudShaderState> for GouraudShader<'_> {
    fn vertex(&self, input: [Vertex; 3]) -> ([Vec3; 3], GouraudShaderState) {
        let mut output = [Vec3::ZERO; 3];
        let mut varying_uv = [Vec2::ZERO; 3];
        let mut light_intensity = [0f32; 3];
        for (i, vert) in input.iter().enumerate() {
            output[i] = {
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

            // TODO transform normal (recalculate them after the transform) for use as normal map and for lighting

            // Calculate the light intensity
            light_intensity[i] = vert.normal.dot(self.light_dir);
        }

        (
            output,
            GouraudShaderState {
                varying_uv,
                light_intensity,
            },
        )
    }

    fn fragment(&self, barycentric_coords: Vec3, state: &GouraudShaderState) -> Option<RGB8> {
        let GouraudShaderState {
            varying_uv,
            light_intensity,
        } = state;

        let uv = varying_uv[0] * barycentric_coords[0]
            + varying_uv[1] * barycentric_coords[1]
            + varying_uv[2] * barycentric_coords[2];

        //TODO use normal map to calculate lighting, instead of barycentric coords
        let weighted_light_intensity = light_intensity[0] * barycentric_coords[0]
            + light_intensity[1] * barycentric_coords[1]
            + light_intensity[2] * barycentric_coords[2];

        let unlit_color = if let Some(ref tex) = self.diffuse_texture {
            tex.data[(tex.height - uv.y as usize) * tex.width + uv.x as usize]
        } else {
            crab_tv::WHITE
        };

        Some(unlit_color.map(|comp| (comp as f32 * weighted_light_intensity) as u8))
    }
}
