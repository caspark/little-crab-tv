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
    #[serde(skip)]
    scene: RenderScene,
    demo_mode_speed: f32,
    #[serde(skip)]
    demo_mode_time_in_scene: f32,
    width: usize,
    height: usize,
    model: PathBuf,
    auto_rotate_camera_speed: f32,
    #[serde(skip)]
    auto_rotate_camera_angle: f32,
    light_dir: Vec3,
    auto_rotate_light_speed: f32,
    #[serde(skip)]
    auto_rotate_light_angle: f32,
    camera_distance: f32,
    camera_look_from: Vec3,
    camera_look_at: Vec3,
    camera_up: Vec3,
    phong_lighting_weights: Vec3,
    use_tangent_space_normal_map: bool,
    shadow_darkness: f32,
    shadow_z_fix: f32,
    ambient_occlusion_passes: usize,
    ambient_occlusion_strength: f32,
    enable_glow_map: bool,
    base_shininess: f32,
    output_filename: String,
    display_actual_size: bool,
    auto_rerender: bool,
}

impl RenderConfig {
    pub(crate) fn image_pixel_count(&self) -> usize {
        self.width * self.height
    }

    pub(crate) fn always_re_render(&self) -> bool {
        self.auto_rotate_light_speed > 0.0
            || self.auto_rotate_camera_speed > 0.0
            || self.demo_mode_speed > 0.0
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

        if self.shadow_darkness < 0.0 {
            bail!("Shadow darkness must be 0.0 or greater");
        } else if self.shadow_darkness > 1.0 {
            bail!("Height must be 1.0 or less");
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
            phong_lighting_weights: self.phong_lighting_weights,
            use_tangent_space_normal_map: self.use_tangent_space_normal_map,
            shadow_darkness: self.shadow_darkness,
            shadow_z_fix: self.shadow_z_fix,
            ambient_occlusion_passes: self.ambient_occlusion_passes,
            ambient_occlusion_strength: self.ambient_occlusion_strength,
            enable_glow_map: self.enable_glow_map,
            base_shininess: self.base_shininess,
        })
    }
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            scene: RenderScene::default(),
            demo_mode_speed: 1.0,
            demo_mode_time_in_scene: 0.0,
            width: 1000,
            height: 1000,
            model: PathBuf::from("assets/head.obj"),
            auto_rotate_camera_speed: 0.1,
            auto_rotate_camera_angle: 0.0,
            light_dir: Vec3::new(0.0, 0.0, 1.0),
            auto_rotate_light_speed: 0.1,
            auto_rotate_light_angle: 0.0,
            camera_distance: 3.0,
            camera_look_from: Vec3::new(0.0, 0.0, 3.0),
            camera_look_at: Vec3::ZERO,
            camera_up: Vec3::new(0.0, 1.0, 0.0),
            phong_lighting_weights: Vec3::new(1.0, 1.0, 0.6),
            use_tangent_space_normal_map: true,
            shadow_darkness: 0.7,
            shadow_z_fix: 5.0,
            ambient_occlusion_passes: 5,
            ambient_occlusion_strength: 2.0,
            enable_glow_map: true,
            base_shininess: 5.0,
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
    phong_lighting_weights: Vec3,
    use_tangent_space_normal_map: bool,
    shadow_darkness: f32,
    shadow_z_fix: f32,
    ambient_occlusion_passes: usize,
    ambient_occlusion_strength: f32,
    enable_glow_map: bool,
    base_shininess: f32,
}

fn main() {
    let app = ui::RendererApp::new();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}
