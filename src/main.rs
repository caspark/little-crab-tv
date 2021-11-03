#![deny(clippy::all)] // make all clippy warnings into errors
#![allow(clippy::many_single_char_names)]

mod ui;

use crab_tv::{canvas::Canvas, Model};
use rgb::RGB8;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
struct RenderConfig {
    image_width: usize,
    image_height: usize,
    model_filename: String,
    output_filename: String,
    display_actual_size: bool,
}

impl RenderConfig {
    pub(crate) fn image_pixel_count(&self) -> usize {
        self.image_width * self.image_height
    }
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            image_width: 1000,
            image_height: 1000,
            model_filename: "models/african_head.obj".to_owned(),
            output_filename: "target/output.png".to_owned(),
            display_actual_size: true,
        }
    }
}

enum RenderCommand {
    Render { config: RenderConfig },
}

enum RenderResult {
    Reset {
        image_width: usize,
        image_height: usize,
    },
    FullImage {
        pixels: Vec<RGB8>,
    },
}

fn main() {
    let (command_tx, command_rx) = flume::unbounded::<RenderCommand>();
    let (result_tx, result_rx) = flume::unbounded::<RenderResult>();

    // start a background thread to handle rendering, but drop its handle so we don't wait for it
    // to finish
    drop(std::thread::spawn(move || {
        run_render_loop(command_rx, result_tx);
    }));

    let app = ui::TemplateApp::new(command_tx, result_rx);
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}

fn run_render_loop(
    render_command_rx: flume::Receiver<RenderCommand>,
    render_result_tx: flume::Sender<RenderResult>,
) {
    loop {
        match render_command_rx.recv() {
            Err(flume::RecvError::Disconnected) => break, // nothing to do, just quit quietly

            Ok(RenderCommand::Render { config }) => {
                render_result_tx
                    .send(RenderResult::Reset {
                        image_height: config.image_height,
                        image_width: config.image_width,
                    })
                    .ok()
                    .expect("sending Reset should succeed");

                let model = Model::load_from_file(config.model_filename)
                    .expect("model filename should exist");

                let mut image = Canvas::new(config.image_width, config.image_height);

                let white = RGB8::new(255, 255, 255);

                for face in model.faces.iter() {
                    for j in 0..3 {
                        let v0 = model.vertices[face.vertices[j]];
                        debug_assert!(
                            face.vertices.len() == 3,
                            "only faces with exactly 3 vertices are supported; found {} vertices",
                            face.vertices.len()
                        );
                        let v1 = model.vertices[face.vertices[(j + 1) % 3]];

                        // this simplistic rendering code assumes that the vertice coordinates are
                        // between -1 and 1, so confirm that assumption
                        debug_assert!(
                            -1.0 <= v0.pos.x && v0.pos.x <= 1.0,
                            "x coordinate out of range: {}",
                            v0.pos.x
                        );
                        debug_assert!(
                            -1.0 <= v0.pos.y && v0.pos.y <= 1.0,
                            "y coordinate out of range: {}",
                            v0.pos.y
                        );
                        debug_assert!(
                            -1.0 <= v1.pos.x && v1.pos.x <= 1.0,
                            "x coordinate out of range: {}",
                            v1.pos.x
                        );
                        debug_assert!(
                            -1.0 <= v1.pos.y && v1.pos.y <= 1.0,
                            "y coordinate out of range: {}",
                            v1.pos.y
                        );
                        let x0 =
                            ((v0.pos.x + 1.0) * (config.image_width as f32 - 1.0) / 2.0) as i32;
                        let y0 =
                            ((v0.pos.y + 1.0) * (config.image_height as f32 - 1.0) / 2.0) as i32;
                        let x1 =
                            ((v1.pos.x + 1.0) * (config.image_width as f32 - 1.0) / 2.0) as i32;
                        let y1 =
                            ((v1.pos.y + 1.0) * (config.image_height as f32 - 1.0) / 2.0) as i32;

                        image.line(x0, y0, x1, y1, white);
                    }
                }

                image.flip_y(); // flip the image so it's rendered in the right orientation

                render_result_tx
                    .send(RenderResult::FullImage {
                        pixels: image.into_pixels(),
                    })
                    .ok()
                    .expect("Sending final image shoud succeed");
            }
        }
    }
}
