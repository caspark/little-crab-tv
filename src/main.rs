#![deny(clippy::all)] // make all clippy warnings into errors
#![allow(clippy::many_single_char_names)]

mod ui;

use crab_tv::canvas::Canvas;
use rgb::RGB8;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
struct RenderConfig {
    image_width: usize,
    image_height: usize,
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
        let aspect_ratio = 16.0 / 9.0;
        let image_width = 100;
        Self {
            image_width,
            image_height: (image_width as f64 / aspect_ratio) as usize,
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

                let mut image = Canvas::new(config.image_width, config.image_height);

                *image.pixel(
                    config.image_width as i32 / 2,
                    config.image_height as i32 / 2,
                ) = RGB8::new(255, 0, 0);

                let white = RGB8::new(255, 255, 255);
                let red = RGB8::new(255, 0, 0);
                let green = RGB8::new(0, 255, 0);
                let blue = RGB8::new(0, 0, 255);

                image.line(13, 20, 80, 40, white);
                image.line(20, 13, 40, 80, red);
                image.line(80, 40, 13, 20, blue);
                image.line(0, 0, 50, 50, RGB8::new(0, 255, 0));

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
