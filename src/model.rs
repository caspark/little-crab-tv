use derive_more::Constructor;
use glam::Vec3;

#[derive(Clone, Copy, Debug, PartialEq, Constructor)]
pub struct Vertex {
    pub pos: Vec3,
}

#[derive(Clone, Debug, Constructor)]
pub struct Face {
    pub vertices: Vec<usize>,
}

#[derive(Clone, Debug)]
pub struct Model {
    pub vertices: Vec<Vertex>,
    pub faces: Vec<Face>,
}

impl Model {
    pub fn load_from_file<S: AsRef<str>>(path: S) -> std::io::Result<Self> {
        use std::io::prelude::*;

        let mut contents = String::new();
        std::fs::File::open(path.as_ref())?.read_to_string(&mut contents)?;

        let mut vertices = Vec::new();
        let mut faces = Vec::new();
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

                    let mut face_vertices = Vec::new();
                    for vertex in parts {
                        let mut vertex_parts = vertex.split('/');
                        let vertex_index = vertex_parts.next().unwrap().parse::<i32>().unwrap();
                        // vertex indices should be 1-based & we ignore negative indices even though
                        // officially they are allowed
                        assert!(
                            vertex_index > 0,
                            "Only positive 1-based indexing is supported for faces vertex indexing"
                        );
                        face_vertices.push((vertex_index - 1) as usize);
                    }

                    debug_assert!(
                        face_vertices.len() == 3,
                        "only faces with exactly 3 vertices are supported; found {} vertices",
                        face_vertices.len()
                    );
                    faces.push(Face::new(face_vertices));
                }
                _ => (), // ignore unknown line type
            }
        }

        Ok(Self { vertices, faces })
    }
}
