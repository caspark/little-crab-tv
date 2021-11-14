use glam::IVec2;
use rgb::RGB8;

use crate::{RenderConfig, RenderScene};
use crab_tv::{Canvas, Model};

pub fn render_scene(image: &mut Canvas, config: &RenderConfig) {
    let white = RGB8::new(255, 255, 255);
    let red = RGB8::new(255, 0, 0);
    let green = RGB8::new(0, 255, 0);
    let blue = RGB8::new(0, 0, 255);

    match config.scene {
        RenderScene::SinglePixel => {
            *image.pixel(
                config.image_width as i32 / 2,
                config.image_height as i32 / 2,
            ) = RGB8::new(255, 0, 0);
        }
        RenderScene::Lines => {
            image.line(IVec2::new(13, 20), IVec2::new(80, 40), white);
            image.line(IVec2::new(20, 13), IVec2::new(40, 80), red);
            image.line(IVec2::new(80, 40), IVec2::new(13, 20), blue);
            image.line(IVec2::new(0, 0), IVec2::new(50, 50), green);
        }
        RenderScene::Wireframe => {
            let model = Model::load_from_file(config.model_filename.as_str())
                .expect("model filename should exist");

            image.wireframe(&model, white);
            image.flip_y(); // flip the image so it's rendered in the right orientation
        }
        RenderScene::Triangles => {
            let t0 = [IVec2::new(10, 70), IVec2::new(50, 160), IVec2::new(70, 80)];
            let t1 = [IVec2::new(180, 50), IVec2::new(150, 1), IVec2::new(70, 180)];
            let t2 = [
                IVec2::new(180, 150),
                IVec2::new(120, 160),
                IVec2::new(130, 180),
            ];
            image.triangle(t0[0], t0[1], t0[2], red);
            image.triangle(t1[0], t1[1], t1[2], white);
            image.triangle(t2[0], t2[1], t2[2], green);
        }
    }
}
