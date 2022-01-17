use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use derive_more::Constructor;
use glam::{Vec2, Vec3};
use rgb::{ComponentMap, RGB8};

#[derive(Clone, Copy, Debug, PartialEq, Constructor)]
pub struct Vertex {
    pub pos: Vec3,
}

#[derive(Clone, Debug, Constructor)]
pub struct FacePoint {
    pub vertices_index: usize,
    pub uv_index: usize,
    pub normals_index: usize,
}

#[derive(Clone, Debug, Constructor)]
pub struct Face {
    pub points: Vec<FacePoint>,
}

type TextureInput = PathBuf;

#[derive(Clone, Debug, Constructor)]
pub struct Texture {
    pub width: usize,
    pub height: usize,
    pub data: Vec<RGB8>,
}

impl Texture {
    fn validate(path: &Path) -> Result<TextureInput> {
        if !path.exists() {
            bail!("Texture file does not exist: {}", path.display());
        }
        Ok(path.to_owned())
    }

    fn load_from_file(path: &TextureInput) -> Result<Self> {
        println!("Loading texture from file: {}", path.display());
        let diffuse_bitmap = lodepng::decode24_file(path)
            .with_context(|| format!("Loading texture from '{}' failed", path.display()))?;
        Ok(Texture::new(
            diffuse_bitmap.width,
            diffuse_bitmap.height,
            diffuse_bitmap.buffer,
        ))
    }

    pub fn get_pixel(&self, uv: Vec2) -> RGB8 {
        let x = uv.x as usize;
        let y = uv.y as usize;
        debug_assert!(x < self.width);
        debug_assert!(y < self.height);

        self.data[(self.height - y as usize) * self.width + x as usize]
    }

    pub fn get_normal(&self, uv: Vec2) -> Vec3 {
        let p = self
            .get_pixel(uv)
            // now normalize to [-1.0, 1.0]
            .map(|comp| comp as f32 / 255.0 * 2.0 - 1.0);
        Vec3::new(p.r, p.g, p.b)
    }

    pub fn get_specular(&self, uv: Vec2) -> f32 {
        // we assume that each of the rgb channels have the same data and arbitrarily pick R to read
        // the specular from
        self.get_pixel(uv).r as f32
    }
}

#[derive(Clone, Debug)]
pub struct ModelInput {
    model: PathBuf,
    diffuse_texture: PathBuf,
    normal_texture_global: PathBuf,
    normal_texture_darboux: PathBuf,
    specular_texture: PathBuf,
}

impl ModelInput {
    pub fn path(&self) -> &Path {
        self.model.as_path()
    }
}

#[derive(Clone, Debug)]
pub struct Model {
    pub vertices: Vec<Vertex>,
    pub vertex_normals: Vec<Vec3>,
    pub faces: Vec<Face>,
    pub texture_coords: Vec<Vec2>,
    pub diffuse_texture: Texture,
    /// Normal texture in global/cartesian coordinate system - should be mostly multicolor
    pub normal_texture_global: Texture,
    /// Normal texture in darboux frame (tangent space) - should be mostly blue
    pub normal_texture_darboux: Texture,
    pub specular_texture: Texture,
}

impl Model {
    pub fn validate(model: &Path) -> Result<ModelInput> {
        let model_ext = model
            .extension()
            .ok_or_else(|| anyhow!("Model file '{:?}' must have an extension", model))?;
        if model_ext != "obj" {
            bail!(
                "Model file '{:?}' must be an Obj file that ends in .obj",
                model
            );
        }

        let diffuse_texture = Texture::validate(model.with_extension("diffuse.png").as_ref())
            .context("Validating diffuse texture failed")?;
        let normal_texture_global =
            Texture::validate(model.with_extension("normals_global.png").as_ref())
                .context("Validating (global space) normal texture failed")?;
        let normal_texture_darboux =
            Texture::validate(model.with_extension("normals_darboux.png").as_ref())
                .context("Validating (darboux frame) normal texture failed")?;
        let specular_texture = Texture::validate(model.with_extension("specular.png").as_ref())
            .context("Validating specular texture failed")?;

        Ok(ModelInput {
            model: model.to_owned(),
            diffuse_texture,
            normal_texture_global,
            normal_texture_darboux,
            specular_texture,
        })
    }

    pub fn load_obj_file(input: &ModelInput) -> Result<Self> {
        use std::io::prelude::*;

        println!("Loading model from file: {}", input.model.display());
        let mut contents = String::new();
        std::fs::File::open(&input.model)
            .with_context(|| "attempting to open model file")?
            .read_to_string(&mut contents)
            .with_context(|| "attempting to read model file")?;

        let mut vertices = Vec::new();
        let mut faces = Vec::new();
        let mut texture_coords = Vec::new();
        let mut vertex_normals = Vec::new();
        for line in contents.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let mut parts = line.split_whitespace();

            let line_type = parts.next().unwrap();
            match line_type {
                "v" => {
                    // vertex, eg: v 0.608654 -0.568839 -0.416318
                    let mut extract_float = || {
                        parts
                            .next()
                            .expect("vertex data point")
                            .parse::<f32>()
                            .expect("vertex float position")
                    };
                    let x = extract_float();
                    let y = extract_float();
                    let z = extract_float();
                    vertices.push(Vertex::new(Vec3::new(x, y, z)));
                }
                "f" => {
                    // face, eg: f 1193/1240/1193 1180/1227/1180 1179/1226/1179
                    let mut vertices = Vec::new();
                    for vertex in parts {
                        let mut vertex_parts = vertex.split('/');
                        let vertices_index = vertex_parts.next().unwrap().parse::<i32>().unwrap();
                        let uvs_index = vertex_parts.next().unwrap().parse::<i32>().unwrap();
                        let normals_index = vertex_parts.next().unwrap().parse::<i32>().unwrap();
                        // vertex indices should be 1-based & we ignore negative indices even though
                        // officially they are allowed
                        assert!(
                            vertices_index > 0,
                            "Only positive 1-based indexing is supported for faces vertex indexing"
                        );
                        assert!(
                            uvs_index > 0,
                            "Only positive 1-based indexing is supported for face texture coordinate indexing"
                        );
                        assert!(
                            normals_index > 0,
                            "Only positive 1-based indexing is supported for face normal indexing"
                        );

                        vertices.push(FacePoint::new(
                            vertices_index as usize - 1,
                            uvs_index as usize - 1,
                            normals_index as usize - 1,
                        ));
                    }
                    debug_assert!(
                        vertices.len() == 3,
                        "only faces with exactly 3 vertices are supported; found {} vertices",
                        vertices.len()
                    );
                    faces.push(Face::new(vertices));
                }
                "vt" => {
                    // triangle texture coordinates, eg: vt  0.532 0.923 0.000
                    let mut extract_float = || {
                        parts
                            .next()
                            .expect("vertex tex coord")
                            .parse::<f32>()
                            .expect("vertex float coord")
                    };
                    let u = extract_float();
                    let v = extract_float();
                    texture_coords.push(Vec2::new(u, v));
                }
                "vn" => {
                    // vertex normal vectors, eg: vn  0.001 0.482 -0.876
                    let mut extract_float = || {
                        parts
                            .next()
                            .expect("vertex normal component")
                            .parse::<f32>()
                            .expect("vertex float component")
                    };
                    let x = extract_float();
                    let y = extract_float();
                    let z = extract_float();
                    vertex_normals.push(Vec3::new(x, y, z));
                }
                _ => (), // ignore unknown line type
            }
        }

        let diffuse_texture = Texture::load_from_file(&input.diffuse_texture)
            .context("Loading diffuse texture failed")?;
        let normal_texture_global = Texture::load_from_file(&input.normal_texture_global)
            .context("Loading (global space) normal texture failed")?;
        let normal_texture_darboux = Texture::load_from_file(&input.normal_texture_darboux)
            .context("Loading (darboux frame) normal texture failed")?;
        let specular_texture = Texture::load_from_file(&input.specular_texture)
            .context("Loading specular texture failed")?;

        Ok(Self {
            vertices,
            vertex_normals,
            faces,
            texture_coords,
            diffuse_texture,
            normal_texture_global,
            normal_texture_darboux,
            specular_texture,
        })
    }
}
