use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use derive_more::Constructor;
use glam::Vec3;
use rgb::RGB8;

#[derive(Clone, Copy, Debug, PartialEq, Constructor)]
pub struct Vertex {
    pub pos: Vec3,
}

#[derive(Clone, Debug, Constructor)]
pub struct Face {
    pub vertices: Vec<usize>,
    pub texture_coords: Vec<usize>,
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

    pub fn get_pixel(&self, x: usize, y: usize) -> RGB8 {
        debug_assert!(x < self.width);
        debug_assert!(y < self.height);
        self.data[y * self.width + x]
    }
}

#[derive(Clone, Debug)]
pub struct ModelInput {
    model: PathBuf,
    diffuse_texture: PathBuf,
}

impl ModelInput {}

#[derive(Clone, Debug)]
pub struct Model {
    pub vertices: Vec<Vertex>,
    pub faces: Vec<Face>,
    pub texture_coords: Vec<Vec3>,
    pub diffuse_texture: Texture,
}

impl Model {
    pub fn validate(model: &Path) -> Result<ModelInput> {
        let model_ext = model
            .extension()
            .ok_or(anyhow!("Model file '{:?}' must have an extension", model))?;
        if model_ext != "obj" {
            bail!(
                "Model file '{:?}' must be an Obj file that ends in .obj",
                model
            );
        }

        let diffuse_texture = Texture::validate(model.with_extension("diffuse.png").as_ref())
            .context("Loading diffuse texture failed")?;

        Ok(ModelInput {
            model: model.to_owned(),
            diffuse_texture,
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
        for line in contents.lines() {
            let line = line.trim();
            if line.len() == 0 {
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
                    let mut vertex_indexes = Vec::new();
                    let mut texture_indexes = Vec::new();
                    let mut face_vertex_count = 0;
                    for vertex in parts {
                        let mut vertex_parts = vertex.split('/');
                        let vertex_index = vertex_parts.next().unwrap().parse::<i32>().unwrap();
                        let texture_index = vertex_parts.next().unwrap().parse::<i32>().unwrap();
                        // vertex indices should be 1-based & we ignore negative indices even though
                        // officially they are allowed
                        assert!(
                            vertex_index > 0,
                            "Only positive 1-based indexing is supported for faces vertex indexing"
                        );
                        assert!(
                            texture_index > 0,
                            "Only positive 1-based indexing is supported for face texture coordinate indexing"
                        );
                        vertex_indexes.push((vertex_index - 1) as usize);
                        texture_indexes.push((vertex_index - 1) as usize);

                        face_vertex_count += 1;
                    }
                    debug_assert!(
                        face_vertex_count == 3,
                        "only faces with exactly 3 vertices are supported; found {} vertices",
                        face_vertex_count
                    );
                    faces.push(Face::new(vertex_indexes, texture_indexes));
                }
                "vt" => {
                    // triangle texture coordinates, eg: vt  0.532 0.923 0.000
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
                    texture_coords.push(Vec3::new(x, y, z));
                }
                _ => (), // ignore unknown line type
            }
        }

        let diffuse_texture = Texture::load_from_file(&input.diffuse_texture)
            .context("Loading diffuse texture failed")?;

        Ok(Self {
            vertices,
            faces,
            texture_coords,
            diffuse_texture,
        })
    }
}
