use glam::IVec2;

use crate::RenderConfig;
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
pub(crate) enum RenderScene {
    FivePixels,
    Lines,
    Wireframe,
    TrianglesOrig,
    TrianglesRefined,
}

pub fn render_scene(image: &mut Canvas, config: &RenderConfig) {
    match config.scene {
        RenderScene::FivePixels => {
            // pixel in the middle
            *image.pixel(config.width as i32 / 2, config.height as i32 / 2) = WHITE;
            // then each of the 4 corners
            *image.pixel(0, config.height as i32 - 1) = RED; // top left
            *image.pixel(config.width as i32 - 1, config.height as i32 - 1) = GREEN; // top right
            *image.pixel(0, 0) = BLUE; // bottom left
            *image.pixel(config.width as i32 - 1, 0) = CYAN; // bottom right
        }
        RenderScene::Lines => {
            image.line(IVec2::new(13, 20), IVec2::new(80, 40), WHITE);
            image.line(IVec2::new(20, 13), IVec2::new(40, 80), RED);
            image.line(IVec2::new(80, 40), IVec2::new(13, 20), BLUE);
            image.line(IVec2::new(0, 0), IVec2::new(50, 50), GREEN);
        }
        RenderScene::Wireframe => {
            let model = Model::load_from_file(config.model_filename.as_str())
                .expect("model filename should exist");

            image.wireframe(&model, WHITE);
        }
        RenderScene::TrianglesOrig => {
            let t0 = [IVec2::new(10, 70), IVec2::new(50, 160), IVec2::new(70, 80)];
            let t1 = [IVec2::new(180, 50), IVec2::new(150, 1), IVec2::new(70, 180)];
            let t2 = [
                IVec2::new(180, 150),
                IVec2::new(120, 160),
                IVec2::new(130, 180),
            ];
            image.triangle_linesweep_orig(t0[0], t0[1], t0[2], RED);
            image.triangle_linesweep_orig(t1[0], t1[1], t1[2], WHITE);
            image.triangle_linesweep_orig(t2[0], t2[1], t2[2], GREEN);
        }
        RenderScene::TrianglesRefined => {
            let t0 = [IVec2::new(10, 70), IVec2::new(50, 160), IVec2::new(70, 80)];
            let t1 = [IVec2::new(180, 50), IVec2::new(150, 1), IVec2::new(70, 180)];
            let t2 = [
                IVec2::new(180, 150),
                IVec2::new(120, 160),
                IVec2::new(130, 180),
            ];
            image.triangle_linesweep_refined(t0[0], t0[1], t0[2], RED);
            image.triangle_linesweep_refined(t1[0], t1[1], t1[2], WHITE);
            image.triangle_linesweep_refined(t2[0], t2[1], t2[2], GREEN);
        }
    }

    image.flip_y();
}
