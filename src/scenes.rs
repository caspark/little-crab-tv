use anyhow::Result;
use glam::{IVec2, Mat4, Vec3};

use crab_tv::{Canvas, Model, ModelShading, BLUE, CYAN, GREEN, RED, WHITE};

#[derive(
    Copy,
    Clone,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    strum::EnumIter,
    PartialEq,
    Eq,
    strum::Display,
)]
#[strum(serialize_all = "title_case")]
pub enum RenderScene {
    FivePixels,
    Lines,
    ModelWireframe,
    TriangleLineSweepVerbose,
    TriangleLineSweepCompact,
    TriangleBarycentric,
    ModelColoredTriangles,
    ModelFlatShaded,
    ModelDepthTested,
    ModelTextured,
    ModelPerspective,
    ModelGouraud,
    CameraMovable,
    ShaderGouraud,
}

pub fn render_scene(
    image: &mut Canvas,
    scene: &RenderScene,
    model: &Model,
    light_dir: Vec3,
    camera_distance: f32,
    camera_look_from: Vec3,
    camera_look_at: Vec3,
    camera_up: Vec3,
) -> Result<()> {
    println!("Rendering scene: {}", scene);

    // projection matrix applies perspective correction
    let perspective_projection_transform = Mat4::from_cols(
        [1.0, 0.0, 0.0, 0.0].into(),
        [0.0, 1.0, 0.0, 0.0].into(),
        [0.0, 0.0, 1.0, -1.0 / camera_distance].into(),
        [0.0, 0.0, 0.0, 1.0].into(),
    );

    match scene {
        RenderScene::FivePixels => {
            // pixel in the middle
            *image.pixel(image.width() as i32 / 2, image.height() as i32 / 2) = WHITE;
            // then each of the 4 corners
            *image.pixel(0, image.height() as i32 - 1) = RED; // top left
            *image.pixel(image.width() as i32 - 1, image.height() as i32 - 1) = GREEN; // top right
            *image.pixel(0, 0) = BLUE; // bottom left
            *image.pixel(image.width() as i32 - 1, 0) = CYAN; // bottom right
        }
        RenderScene::Lines => {
            image.line(IVec2::new(13, 20), IVec2::new(80, 40), WHITE);
            image.line(IVec2::new(20, 13), IVec2::new(40, 80), RED);
            image.line(IVec2::new(80, 40), IVec2::new(13, 20), BLUE);
            image.line(IVec2::new(0, 0), IVec2::new(50, 50), GREEN);
        }
        RenderScene::ModelWireframe => {
            image.model_wireframe(&model, WHITE);
        }
        RenderScene::TriangleLineSweepVerbose => {
            let t0 = [IVec2::new(10, 70), IVec2::new(50, 160), IVec2::new(70, 80)];
            let t1 = [IVec2::new(180, 50), IVec2::new(150, 1), IVec2::new(70, 180)];
            let t2 = [
                IVec2::new(180, 150),
                IVec2::new(120, 160),
                IVec2::new(130, 180),
            ];
            image.triangle_linesweep_verbose(&t0, RED);
            image.triangle_linesweep_verbose(&t1, WHITE);
            image.triangle_linesweep_verbose(&t2, GREEN);
        }
        RenderScene::TriangleLineSweepCompact => {
            let t0 = [IVec2::new(10, 70), IVec2::new(50, 160), IVec2::new(70, 80)];
            let t1 = [IVec2::new(180, 50), IVec2::new(150, 1), IVec2::new(70, 180)];
            let t2 = [
                IVec2::new(180, 150),
                IVec2::new(120, 160),
                IVec2::new(130, 180),
            ];
            image.triangle_linesweep_compact(&t0, RED);
            image.triangle_linesweep_compact(&t1, WHITE);
            image.triangle_linesweep_compact(&t2, GREEN);
        }
        RenderScene::TriangleBarycentric => {
            let t0 = [IVec2::new(10, 70), IVec2::new(50, 160), IVec2::new(70, 80)];
            let t1 = [IVec2::new(180, 50), IVec2::new(150, 1), IVec2::new(70, 180)];
            let t2 = [
                IVec2::new(180, 150),
                IVec2::new(120, 160),
                IVec2::new(130, 180),
            ];
            image.triangle_barycentric(&t0, RED);
            image.triangle_barycentric(&t1, WHITE);
            image.triangle_barycentric(&t2, GREEN);
        }
        RenderScene::ModelColoredTriangles => {
            image.model_colored_triangles(&model);
        }
        RenderScene::ModelFlatShaded => {
            image.model_fixed_function(&model, light_dir, ModelShading::FlatOnly, None);
        }
        RenderScene::ModelDepthTested => {
            image.model_fixed_function(&model, light_dir, ModelShading::DepthTested, None);
        }
        RenderScene::ModelTextured => {
            image.model_fixed_function(&model, light_dir, ModelShading::Textured, None)
        }
        RenderScene::ModelPerspective => image.model_fixed_function(
            &model,
            light_dir,
            ModelShading::Textured,
            Some(perspective_projection_transform),
        ),
        RenderScene::ModelGouraud => image.model_fixed_function(
            &model,
            light_dir,
            ModelShading::Gouraud,
            Some(perspective_projection_transform),
        ),
        RenderScene::CameraMovable => {
            let model_view_transform =
                look_at_transform(camera_look_from, camera_look_at, camera_up);
            image.model_fixed_function(
                &model,
                light_dir,
                ModelShading::Gouraud,
                Some(perspective_projection_transform * model_view_transform),
            )
        }
        RenderScene::ShaderGouraud => {
            let viewport = viewport_transform(
                image.width() as f32 / 8.0,
                image.height() as f32 / 8.0,
                image.width() as f32 * 3.0 / 4.0,
                image.height() as f32 * 3.0 / 4.0,
            );
            let model_view_transform =
                look_at_transform(camera_look_from, camera_look_at, camera_up);

            let mut shader = crate::shaders::GouraudShader::new(
                viewport * perspective_projection_transform * model_view_transform,
                light_dir,
                Some(&model.diffuse_texture),
            );

            image.model_shader(&model, &mut shader);
        }
    }

    image.flip_y();

    Ok(())
}

fn look_at_transform(eye: Vec3, center: Vec3, up: Vec3) -> Mat4 {
    let z = (eye - center).normalize();
    let x = up.cross(z).normalize();
    let y = z.cross(x).normalize();
    let mut minv = Mat4::IDENTITY;
    let mut tr = Mat4::IDENTITY;
    for i in 0..3 {
        minv.col_mut(i)[0] = x[i];
        minv.col_mut(i)[1] = y[i];
        minv.col_mut(i)[2] = z[i];
        tr.col_mut(3)[i] = -center[i];
    }
    minv * tr
}

// viewport matrix resizes/repositions the result to fit on screen
fn viewport_transform(x: f32, y: f32, w: f32, h: f32) -> Mat4 {
    let depth = 255.0;
    Mat4::from_cols(
        [w / 2.0, 0.0, 0.0, 0.0].into(),
        [0.0, h / 2.0, 0.0, 0.0].into(),
        [0.0, 0.0, depth / 2.0, 0.0].into(),
        [x + w / 2.0, y + h / 2.0, depth / 2.0, 1.0].into(),
    )
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use strum::IntoEnumIterator;

    use super::*;

    #[test]
    fn every_scene_should_render_without_errors() -> Result<()> {
        for scene in RenderScene::iter() {
            let mut image = Canvas::new(200, 200);
            println!("Rendering scene: {:?}", scene);
            render_scene(
                &mut image,
                &scene,
                &Model::load_obj_file(&Model::validate(
                    Path::new("assets/african_head.obj").as_ref(),
                )?)
                .expect("model load should succeed"),
                Vec3::new(0.0, 0.0, -1.0),
                3.0,
                Vec3::new(0.0, 0.0, 3.0),
                Vec3::ZERO,
                Vec3::new(0.0, 1.0, 0.0),
            )?;
        }
        Ok(())
    }
}
