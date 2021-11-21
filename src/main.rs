#![deny(clippy::all)] // make all clippy warnings into errors
#![allow(clippy::many_single_char_names)]

mod scenes;
mod ui;

use std::path::{Path, PathBuf};

use crate::scenes::{render_scene, RenderScene};
use anyhow::{bail, Context, Result};
use crab_tv::{Canvas, Model, ModelInput};
use glam::Vec3;
use rgb::RGB8;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct RenderConfig {
    scene: RenderScene,
    width: usize,
    height: usize,
    model: PathBuf,
    light_dir: Vec3,
    output_filename: String,
    display_actual_size: bool,
    auto_rerender: bool,
}

impl RenderConfig {
    pub(crate) fn image_pixel_count(&self) -> usize {
        self.width * self.height
    }

    pub(crate) fn validate(&self) -> Result<RenderInput> {
        if self.width < 200 {
            bail!("Width must be 200 or greater");
        } else if self.width > 5000 {
            bail!("Width must be 5000 or less");
        }
        if self.height < 200 {
            bail!("Height must be 200 or greater");
        } else if self.height > 5000 {
            bail!("Height must be 5000 or less");
        }

        let model_input = Model::validate(&self.model)
            .with_context(|| format!("Failed to load model from {}", self.model.display()))?;

        Ok(RenderInput {
            scene: self.scene,
            width: self.width,
            height: self.height,
            model_input,
            light_dir: self.light_dir,
        })
    }
}
impl RenderConfig {
    #![allow(unused)]

    pub(crate) fn scene(&mut self, scene: RenderScene) -> &mut Self {
        self.scene = scene;
        self
    }

    pub(crate) fn dimensions(&mut self, width: usize, height: usize) -> &mut Self {
        self.width = width;
        self.height = height;
        self
    }

    pub(crate) fn model(&mut self, model: &Path) -> &mut Self {
        self.model = model.to_owned();
        self
    }
    pub(crate) fn output_filename(&mut self, output_filename: String) -> &mut Self {
        self.output_filename = output_filename;
        self
    }

    pub(crate) fn display_actual_size(&mut self, display_actual_size: bool) -> &mut Self {
        self.display_actual_size = display_actual_size;
        self
    }
}

impl Default for RenderConfig {
    fn default() -> Self {
        use strum::IntoEnumIterator;

        Self {
            scene: RenderScene::iter().last().unwrap(),
            width: 400,
            height: 400,
            model: PathBuf::from("assets/african_head"),
            light_dir: Vec3::new(0.0, 0.0, -1.0),
            output_filename: "target/output.png".to_owned(),
            display_actual_size: true,
            auto_rerender: true,
        }
    }
}

#[derive(Clone, Debug)]
pub struct RenderInput {
    scene: RenderScene,
    width: usize,
    height: usize,
    model_input: ModelInput,
    light_dir: Vec3,
}

enum RenderCommand {
    Render { input: RenderInput },
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

            Ok(RenderCommand::Render { input }) => {
                render_result_tx
                    .send(RenderResult::Reset {
                        image_height: input.height,
                        image_width: input.width,
                    })
                    .ok()
                    .expect("sending Reset should succeed");

                let mut image = Canvas::new(input.width, input.height);

                let model = Model::load_obj_file(&input.model_input).expect("Failed to load model");

                render_scene(&mut image, &input.scene, &model, input.light_dir).unwrap();

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
