use glam::IVec2;

use crab_tv::{Canvas, Model, BLUE, CYAN, GREEN, RED, WHITE};

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
}

pub fn render_scene(image: &mut Canvas, scene: &RenderScene, model_filename: &str) {
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
            println!("Loading model: {}", model_filename);
            let model = Model::load_from_file(model_filename).expect("model filename should exist");

            image.wireframe(&model, WHITE);
        }
        RenderScene::TriangleLineSweepVerbose => {
            let t0 = [IVec2::new(10, 70), IVec2::new(50, 160), IVec2::new(70, 80)];
            let t1 = [IVec2::new(180, 50), IVec2::new(150, 1), IVec2::new(70, 180)];
            let t2 = [
                IVec2::new(180, 150),
                IVec2::new(120, 160),
                IVec2::new(130, 180),
            ];
            image.triangle_linesweep_verbose(t0[0], t0[1], t0[2], RED);
            image.triangle_linesweep_verbose(t1[0], t1[1], t1[2], WHITE);
            image.triangle_linesweep_verbose(t2[0], t2[1], t2[2], GREEN);
        }
        RenderScene::TriangleLineSweepCompact => {
            let t0 = [IVec2::new(10, 70), IVec2::new(50, 160), IVec2::new(70, 80)];
            let t1 = [IVec2::new(180, 50), IVec2::new(150, 1), IVec2::new(70, 180)];
            let t2 = [
                IVec2::new(180, 150),
                IVec2::new(120, 160),
                IVec2::new(130, 180),
            ];
            image.triangle_linesweep_compact(t0[0], t0[1], t0[2], RED);
            image.triangle_linesweep_compact(t1[0], t1[1], t1[2], WHITE);
            image.triangle_linesweep_compact(t2[0], t2[1], t2[2], GREEN);
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
            println!("Loading model: {}", model_filename);
            let model = Model::load_from_file(model_filename).expect("model filename should exist");

            image.colored_triangles(&model);
        }
    }

    image.flip_y();
}

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;

    use super::*;

    #[test]
    fn every_scene_should_render_without_errors() {
        for scene in RenderScene::iter() {
            let mut image = Canvas::new(200, 200);
            println!("Rendering scene: {:?}", scene);
            render_scene(&mut image, &scene, "models/african_head.obj");
        }
    }
}
