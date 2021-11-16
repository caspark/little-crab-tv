#![deny(clippy::all)] // make all clippy warnings into errors
#![allow(clippy::many_single_char_names)]

mod scenes;
mod ui;

use crate::scenes::{render_scene, RenderScene};
use crab_tv::Canvas;
use rgb::RGB8;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct RenderConfig {
    scene: RenderScene,
    width: usize,
    height: usize,
    model_filename: String,
    output_filename: String,
    display_actual_size: bool,
}

impl RenderConfig {
    pub(crate) fn image_pixel_count(&self) -> usize {
        self.width * self.height
    }
}

impl Default for RenderConfig {
    fn default() -> Self {
        use strum::IntoEnumIterator;

        Self {
            scene: RenderScene::iter().next().unwrap(),
            width: 400,
            height: 400,
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
                        image_height: config.height,
                        image_width: config.width,
                    })
                    .ok()
                    .expect("sending Reset should succeed");

                let mut image = Canvas::new(config.width, config.height);
                render_scene(&mut image, &config);

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
