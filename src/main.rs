#![deny(clippy::all)] // make all clippy warnings into errors
#![allow(clippy::many_single_char_names)]
#![allow(clippy::needless_range_loop)]

mod scenes;
mod shaders;
mod ui;

use std::path::PathBuf;

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
    camera_distance: f32,
    camera_look_from: Vec3,
    camera_look_at: Vec3,
    camera_up: Vec3,
    ambient_occlusion_passes: usize,
    ambient_occlusion_strength: f32,
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

        if self.camera_look_from == self.camera_look_at {
            bail!("Camera's 'look from' position must not be the same as its 'look at' position");
        }

        Ok(RenderInput {
            scene: self.scene,
            width: self.width,
            height: self.height,
            model_input,
            light_dir: self.light_dir,
            camera_perspective_dist: self.camera_distance,
            camera_look_from: self.camera_look_from,
            camera_look_at: self.camera_look_at,
            camera_up: self.camera_up,
            ambient_occlusion_passes: self.ambient_occlusion_passes,
            ambient_occlusion_strength: self.ambient_occlusion_strength,
        })
    }
}

impl Default for RenderConfig {
    fn default() -> Self {
        use strum::IntoEnumIterator;

        Self {
            scene: RenderScene::iter().last().unwrap(),
            width: 1000,
            height: 1000,
            model: PathBuf::from("assets/african_head.obj"),
            light_dir: Vec3::new(0.0, 0.0, 1.0),
            camera_distance: 3.0,
            camera_look_from: Vec3::new(0.0, 0.0, 3.0),
            camera_look_at: Vec3::ZERO,
            camera_up: Vec3::new(0.0, 1.0, 0.0),
            ambient_occlusion_passes: 5,
            ambient_occlusion_strength: 2.0,
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
    camera_perspective_dist: f32,
    camera_look_from: Vec3,
    camera_look_at: Vec3,
    camera_up: Vec3,
    ambient_occlusion_passes: usize,
    ambient_occlusion_strength: f32,
}

fn main() {
    let app = ui::RendererApp::new();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}
