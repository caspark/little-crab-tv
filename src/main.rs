#![deny(clippy::all)] // make all clippy warnings into errors
#![allow(clippy::many_single_char_names)]

mod scenes;
mod ui;

use std::path::{Path, PathBuf};

use crate::scenes::RenderScene;
use anyhow::{bail, Context, Result};
use crab_tv::{Model, ModelInput};
use glam::Vec3;

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
            model: PathBuf::from("assets/african_head.obj"),
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

fn main() {
    let app = ui::RendererApp::new();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}
