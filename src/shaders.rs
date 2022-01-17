use glam::{Mat3, Mat4, Vec2, Vec3, Vec4};

use crab_tv::{Canvas, Shader, Texture, Vertex};
use rgb::{ComponentMap, RGB8};

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
        let mut varying_tri = Mat3::ZERO;
        let mut varying_uv = [Vec2::ZERO; 3];
        let mut varying_light_intensity = [0f32; 3];
        for (i, vert) in input.iter().enumerate() {
            *varying_tri.col_mut(i) = {
                // Transform the vertex position
                // step 1 - embed into 4D space by converting to homogeneous coordinates
                let mut vec4: Vec4 = (vert.position, 1.0).into();
                // step 2 - multiply with projection & viewport matrices to correct perspective
                vec4 = self.vertex_transform * vec4;
                // step 3 - divide by w to reproject into 3d screen coordinates
                Vec3::new(vec4.x / vec4.w, vec4.y / vec4.w, vec4.z / vec4.w)
            };

            // Transform the vertex texture coordinates based on the texture we have
            varying_uv[i] = if let Some(texture) = self.diffuse_texture {
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
            varying_tri,
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

        let unlit_color = if let Some(tex) = self.diffuse_texture {
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
    /// normal texture must be in global coordinates (not tangent space)
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

#[derive(Clone, Debug)]
pub enum NormalMap<'t> {
    GlobalSpace(&'t Texture),
    TangentSpace(&'t Texture),
}

/// The output of a depth pass rendered from the perspective of a light source, plus the matrix used
/// to undo that transformation.
#[derive(Clone, Debug)]
pub struct PhongShadowInput {
    // transform framebuffer screen coordinates to shadowbuffer screen coordinates for shadows
    uniform_m_shadow: Mat4,
    shadow_buffer: Canvas,
    shadow_multiplier: f32,
    // Require shadows to be this much longer (deeper), to avoid z-fighting
    shadow_z_fix: f32,
}

impl PhongShadowInput {
    pub fn new(
        uniform_m_shadow: Mat4,
        shadow_buffer: Canvas,
        shadow_darkness: f32,
        shadow_z_fix: f32,
    ) -> Self {
        Self {
            uniform_m_shadow,
            shadow_buffer,
            shadow_multiplier: 1.0 - shadow_darkness,
            shadow_z_fix,
        }
    }
}

pub struct PhongShaderState {
    varying_tri: Mat3,
    varying_nrm: Mat3,
    varying_uv: [Vec2; 3],
}

/// Phong shader renders using ambient/diffuse/specular lighting model, with normals rendered using
/// a tangent space normal map.
#[derive(Clone, Debug)]
pub struct PhongShader<'t> {
    viewport: Mat4,
    /// projection matrix * modelview matrix
    uniform_m: Mat4,
    /// projection matrix * modelview matrix then inverted & transposed, for correcting normals
    uniform_mit: Mat4,
    light_dir: Vec3,
    /// Ambient, diffuse, specular lighting weights
    phong_lighting_weights: Vec3,
    diffuse_texture: &'t Texture,
    /// normal texture must be in tangent space coordinates
    normal_texture: NormalMap<'t>,
    specular_texture: &'t Texture,
    shadows: Option<PhongShadowInput>,
}

impl<'t> PhongShader<'t> {
    pub fn new(
        viewport: Mat4,
        uniform_m: Mat4,
        light_dir: Vec3,
        phong_lighting_weights: Vec3,
        diffuse_texture: &'t Texture,
        normal_texture: NormalMap<'t>,
        specular_texture: &'t Texture,
        shadows: Option<PhongShadowInput>,
    ) -> PhongShader<'t> {
        Self {
            viewport,
            uniform_m,
            uniform_mit: uniform_m.inverse().transpose(),
            light_dir,
            phong_lighting_weights,
            diffuse_texture,
            normal_texture,
            specular_texture,
            shadows,
        }
    }
}

impl Shader<PhongShaderState> for PhongShader<'_> {
    fn vertex(&self, input: [Vertex; 3]) -> (Mat3, PhongShaderState) {
        let mut varying_nrm = Mat3::ZERO;
        let mut varying_tri = Mat3::ZERO;
        let mut varying_uv = [Vec2::ZERO; 3];
        for (i, vert) in input.iter().enumerate() {
            *varying_nrm.col_mut(i) = self.uniform_mit.transform_vector3(vert.normal.normalize());

            *varying_tri.col_mut(i) =
                (self.viewport * self.uniform_m).project_point3(vert.position);

            varying_uv[i] = Vec2::new(
                vert.uv.x * self.diffuse_texture.width as f32,
                vert.uv.y * self.diffuse_texture.height as f32,
            );
        }

        (
            varying_tri,
            PhongShaderState {
                varying_nrm,
                varying_tri,
                varying_uv,
            },
        )
    }

    fn fragment(&self, barycentric_coords: Vec3, state: &PhongShaderState) -> Option<RGB8> {
        let PhongShaderState {
            varying_tri,
            varying_uv,
            varying_nrm,
        } = *state;

        let uv = varying_uv[0] * barycentric_coords[0]
            + varying_uv[1] * barycentric_coords[1]
            + varying_uv[2] * barycentric_coords[2];

        // calculate normal for this fragment using the normal texture
        let n = match self.normal_texture {
            NormalMap::GlobalSpace(normal_texture) => self
                .uniform_mit
                .project_point3(normal_texture.get_normal(uv))
                .normalize(),
            NormalMap::TangentSpace(normal_texture) => {
                let bn = (varying_nrm * barycentric_coords).normalize();

                let a_inverse = {
                    let mut a = Mat3::ZERO;
                    *a.col_mut(0) = varying_tri.col(1) - varying_tri.col(0);
                    *a.col_mut(1) = varying_tri.col(2) - varying_tri.col(0);
                    *a.col_mut(2) = bn;
                    a.transpose().inverse()
                };

                let i = a_inverse
                    * Vec3::new(
                        varying_uv[1].x - varying_uv[0].x,
                        varying_uv[2].x - varying_uv[0].x,
                        0.0,
                    );
                let j = a_inverse
                    * Vec3::new(
                        varying_uv[1].y - varying_uv[0].y,
                        varying_uv[2].y - varying_uv[0].y,
                        0.0,
                    );

                let b = {
                    let mut b = Mat3::ZERO;
                    *b.col_mut(0) = i.normalize();
                    *b.col_mut(1) = j.normalize();
                    *b.col_mut(2) = bn;
                    b
                };

                (b * normal_texture.get_normal(uv)).normalize()
            }
        };
        let l = self.uniform_m.project_point3(self.light_dir).normalize();
        let r = (n * (n.dot(l) * 2.0) - l).normalize(); // reflected light

        let unlit_color = self.diffuse_texture.get_pixel(uv);

        // calculate lighting intensity for this pixel
        let ambient_intensity = 1.0;
        let diffuse_intensity = crab_tv::yolo_max(0.0, n.dot(self.light_dir));
        let specular_intensity =
            crab_tv::yolo_max(0.0, r.z).powf(self.specular_texture.get_specular(uv));

        // check if this pixel is shadowed according to the shadow buffer
        let shadow_multiplier = if let Some(PhongShadowInput {
            uniform_m_shadow,
            ref shadow_buffer,
            shadow_multiplier,
            shadow_z_fix,
        }) = &self.shadows
        {
            let uniform_m_shadow = uniform_m_shadow.to_owned();

            // look up corresponding point in the shadow buffer
            let sb_p = {
                let p = uniform_m_shadow * (varying_tri * barycentric_coords).extend(1.0);
                (p / p.w).truncate() // convert from homogenous coordinates back to vec3
            };
            let shaded = (shadow_buffer.pixel(sb_p.x as i32, sb_p.y as i32).r as f32)
                >= sb_p.z + shadow_z_fix;
            if shaded {
                *shadow_multiplier
            } else {
                1.0
            }
        } else {
            1.0
        };

        // phong shading weights of each light component
        let ambient_weight = self.phong_lighting_weights.x;
        let diffuse_weight = self.phong_lighting_weights.y;
        let specular_weight = self.phong_lighting_weights.z;

        Some(unlit_color.map(|comp| {
            (ambient_weight * ambient_intensity
                + (comp as f32 * shadow_multiplier)
                    * (diffuse_weight * diffuse_intensity + specular_weight * specular_intensity))
                as u8
        }))
    }
}

type UnlitShaderState = [Vec2; 3];

/// A shader that renders a texture but doesn't do any lighting
#[derive(Clone, Debug)]
pub struct UnlitShader<'t> {
    vertex_transform: Mat4,
    texture: &'t Texture,
}

impl<'t> UnlitShader<'t> {
    pub fn new(
        viewport: Mat4,
        uniform_m: Mat4, // projection matrix * modelview matrix
        texture: &'t Texture,
    ) -> UnlitShader<'t> {
        Self {
            vertex_transform: viewport * uniform_m,
            texture,
        }
    }
}

impl Shader<UnlitShaderState> for UnlitShader<'_> {
    fn vertex(&self, input: [Vertex; 3]) -> (Mat3, UnlitShaderState) {
        let mut varying_tri = Mat3::ZERO;
        let mut varying_uv = [Vec2::ZERO; 3];
        for (i, vert) in input.iter().enumerate() {
            *varying_tri.col_mut(i) = self.vertex_transform.project_point3(vert.position);

            varying_uv[i] = Vec2::new(
                vert.uv.x * self.texture.width as f32,
                vert.uv.y * self.texture.height as f32,
            )
        }

        (varying_tri, varying_uv)
    }

    fn fragment(&self, barycentric_coords: Vec3, varying_uv: &UnlitShaderState) -> Option<RGB8> {
        let uv = varying_uv[0] * barycentric_coords[0]
            + varying_uv[1] * barycentric_coords[1]
            + varying_uv[2] * barycentric_coords[2];

        let unlit_color = self.texture.get_pixel(uv);

        Some(unlit_color)
    }
}

type DepthVaryingTri = Mat3;

/// Depth shader, used for calculating shadows
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
        let mut varying_tri = Mat3::ZERO;
        for (i, vert) in input.iter().enumerate() {
            *varying_tri.col_mut(i) =
                (self.viewport * self.uniform_m).project_point3(vert.position);
        }

        (varying_tri, varying_tri)
    }

    fn fragment(&self, barycentric_coords: Vec3, varying_tri: &DepthVaryingTri) -> Option<RGB8> {
        let p = (*varying_tri) * barycentric_coords;
        let depth_scaled = p.z / crab_tv::DEPTH_MAX;
        Some(crab_tv::WHITE.map(|c| (c as f32 * depth_scaled) as u8))
    }
}

/// Shades all fragments of the model as one uniform color
#[derive(Clone, Debug)]
pub struct PureColorShader {
    viewport: Mat4,
    /// projection matrix * modelview matrix
    uniform_m: Mat4,
}

impl PureColorShader {
    pub fn new(viewport: Mat4, uniform_m: Mat4) -> PureColorShader {
        Self {
            viewport,
            uniform_m,
        }
    }
}

impl Shader<()> for PureColorShader {
    fn vertex(&self, input: [Vertex; 3]) -> (Mat3, ()) {
        let mut varying_tri = Mat3::ZERO;
        for (i, vert) in input.iter().enumerate() {
            *varying_tri.col_mut(i) =
                (self.viewport * self.uniform_m).project_point3(vert.position);
        }

        (varying_tri, ())
    }

    fn fragment(&self, _barycentric_coords: Vec3, _: &()) -> Option<RGB8> {
        Some(crab_tv::WHITE)
    }
}
