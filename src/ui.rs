use std::path::PathBuf;

use crab_tv::{Canvas, Model};
use eframe::{
    egui::{self, TextureId},
    epi,
};
use glam::Vec3;
use rgb::RGB8;
use strum::IntoEnumIterator;

use crate::{RenderConfig, RenderInput, RenderScene};

#[derive(Debug, Default)]
struct UiData {
    last_render_width: usize,
    last_render_height: usize,
    last_render_pixels: Vec<RGB8>,
    last_render_tex: Option<TextureId>,
}

impl UiData {
    fn new(width: usize, height: usize) -> Self {
        Self {
            last_render_width: width,
            last_render_height: height,
            last_render_pixels: vec![RGB8 { r: 0, g: 0, b: 0 }; width * height],
            ..Default::default()
        }
    }

    fn clear_texture(&mut self, tex_allocator: &mut dyn eframe::epi::TextureAllocator) {
        if let Some(existing_tex) = self.last_render_tex {
            tex_allocator.free(existing_tex);
            self.last_render_tex = None;
        }
    }

    fn store_image(
        &mut self,
        pixels: &[RGB8],
        tex_allocator: &mut dyn eframe::epi::TextureAllocator,
    ) {
        assert_eq!(
            pixels.len(),
            self.last_render_width * self.last_render_height
        );

        self.last_render_pixels = pixels.iter().copied().collect();

        if let Some(existing_tex) = self.last_render_tex {
            tex_allocator.free(existing_tex);
        }
        let tex_pixels = self
            .last_render_pixels
            .iter()
            .map(|rgb| egui::Color32::from_rgba_premultiplied(rgb.r, rgb.g, rgb.b, 255))
            .collect::<Vec<_>>();
        self.last_render_tex = Some(tex_allocator.alloc_srgba_premultiplied(
            (self.last_render_width, self.last_render_height),
            &tex_pixels,
        ));
    }

    fn save_output_to_file(&self, output_filename: &str) {
        // make sure we got all the data we should have
        assert_eq!(
            self.last_render_pixels.len(),
            self.last_render_width * self.last_render_height
        );

        print!(
            "Saving completed image to disk at {} in PNG format...",
            output_filename
        );
        lodepng::encode_file(
            output_filename,
            &self.last_render_pixels,
            self.last_render_width,
            self.last_render_height,
            lodepng::ColorType::RGB,
            8,
        )
        .expect("Encoding result and saving to disk failed");

        println!(" done saving.");
    }
}

#[derive(Debug)]
pub struct RendererApp {
    config: RenderConfig,
    data: Option<UiData>,
    cached_model: Option<(PathBuf, Model)>,
}

impl RendererApp {
    pub(crate) fn new() -> Self {
        RendererApp {
            config: Default::default(),
            data: Default::default(),
            cached_model: None,
        }
    }

    fn trigger_render(
        &mut self,
        input: RenderInput,
        tex_allocator: &mut dyn eframe::epi::TextureAllocator,
    ) {
        println!(
            "Triggering render of {width}x{height} image (total {count} pixels)",
            width = self.config.width,
            height = self.config.height,
            count = self.config.image_pixel_count(),
        );

        // reset UI state
        if let Some(ref mut d) = self.data {
            d.clear_texture(tex_allocator);
        }
        self.data = Some(UiData::new(self.config.width, self.config.height));

        // render new image
        let mut image = Canvas::new(input.width, input.height);

        let model_cache = &mut self.cached_model;
        if let Some((path, _)) = model_cache {
            if path != input.model_input.path() {
                model_cache.take();
            }
        }
        if model_cache.is_none() {
            model_cache.replace((
                input.model_input.path().to_owned(),
                Model::load_obj_file(&input.model_input).expect("Failed to load model"),
            ));
        }
        let model = &self
            .cached_model
            .as_ref()
            .expect("model should be loaded")
            .1;

        crate::scenes::render_scene(
            &mut image,
            &input.scene,
            model,
            input.light_dir,
            input.camera_perspective_dist,
            input.camera_look_from,
            input.camera_look_at,
            input.camera_up,
            input.use_tangent_space_normal_map,
            input.ambient_occlusion_passes,
            input.ambient_occlusion_strength,
        )
        .unwrap();

        let data = self
            .data
            .as_mut()
            .expect("ui data must be present for storing pixels");

        data.store_image(image.into_pixels().as_slice(), tex_allocator);
    }
}

impl epi::App for RendererApp {
    fn name(&self) -> &str {
        "Crab TV"
    }

    /// Called once before the first frame.
    fn setup(
        &mut self,
        _ctx: &egui::CtxRef,
        frame: &mut epi::Frame<'_>,
        _storage: Option<&dyn epi::Storage>,
    ) {
        // Load previous app state (if any).
        if let Some(storage) = _storage {
            self.config = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
        }

        if let Ok(input) = self.config.validate() {
            self.trigger_render(input, frame.tex_allocator());
        }
    }

    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, &self.config);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        egui::SidePanel::left("config_panel")
            // .resizable(false)
            .show(ctx, |ui| {
                egui::ScrollArea::auto_sized().show(ui, |ui| {
                    let config_before = self.config.clone();

                    ui.spacing_mut().item_spacing = egui::Vec2::new(8.0, 8.0);

                    ui.heading("Render Configuration");
                    egui::warn_if_debug_build(ui);
                    ui.end_row();

                    ui.collapsing("Reset to default", |ui| {
                        if ui.button("Load default configuration").clicked() {
                            self.config = RenderConfig::default();
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Scene");
                        ui.vertical(|ui| {
                            for scene in RenderScene::iter() {
                                ui.radio_value(&mut self.config.scene, scene, format!("{}", scene));
                            }
                        });
                    });
                    ui.end_row();

                    ui.collapsing("Display options", |ui| {
                        ui.checkbox(
                            &mut self.config.display_actual_size,
                            "Display render at actual 1:1 size",
                        );
                    });

                    ui.collapsing("Render options", |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Image filename");
                            path_edit_singleline(ui, &mut self.config.model);
                        });
                        ui.end_row();

                        ui.add(
                            egui::Slider::new(&mut self.config.width, 200..=1000)
                                .suffix("px")
                                .text("Image width"),
                        );
                        ui.end_row();

                        ui.add(
                            egui::Slider::new(&mut self.config.height, 200..=1000)
                                .suffix("px")
                                .text("Image height"),
                        );
                        ui.end_row();

                        let light_dir_before = self.config.light_dir;
                        vec3_editor(ui, "Light Dir", &mut self.config.light_dir);
                        if light_dir_before != self.config.light_dir {
                            // only normalize if the chosen light direction has changed, otherwise
                            // this will cause a render loop for certain floating point values
                            self.config.light_dir = self.config.light_dir.normalize_or_zero();
                        }
                        ui.end_row();

                        vec3_editor(ui, "Camera look from", &mut self.config.camera_look_from);
                        ui.end_row();

                        vec3_editor(ui, "Camera look at", &mut self.config.camera_look_at);
                        ui.end_row();

                        let camera_up_before = self.config.camera_up;
                        vec3_editor(ui, "Camera up dir", &mut self.config.camera_up);
                        if camera_up_before != self.config.camera_up {
                            self.config.camera_up = self
                                .config
                                .camera_up
                                .try_normalize()
                                .unwrap_or_else(|| RenderConfig::default().camera_up);
                        }
                        ui.end_row();

                        ui.add(
                            egui::Slider::new(&mut self.config.camera_distance, 1.0..=10.0)
                                .text("Camera perspective distance"),
                        );
                        ui.end_row();

                        ui.checkbox(
                            &mut self.config.use_tangent_space_normal_map,
                            "Use tangent space (rather than global) normal map",
                        );
                        ui.end_row();

                        ui.add(
                            egui::Slider::new(&mut self.config.ambient_occlusion_passes, 1..=15)
                                .text("Ambient occlusion passes"),
                        );
                        ui.end_row();
                        ui.add(
                            egui::Slider::new(
                                &mut self.config.ambient_occlusion_strength,
                                1.0..=10.0,
                            )
                            .text("Ambient occlusion strength"),
                        );
                        ui.end_row();
                    });

                    ui.collapsing("Save render", |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Path");
                            ui.text_edit_singleline(&mut self.config.output_filename);
                            if let Some(ref data) = self.data {
                                let button = egui::widgets::Button::new("Save");
                                if ui.add(button).clicked() {
                                    data.save_output_to_file(self.config.output_filename.as_ref());
                                }
                            }
                        });
                        ui.end_row();
                    });

                    ui.checkbox(&mut self.config.auto_rerender, "Re-render on config change");
                    ui.end_row();

                    match self.config.validate() {
                        Ok(input) => {
                            if self.config.auto_rerender {
                                if config_before != self.config {
                                    println!("Configuration change detected - auto-rerendering!");
                                    self.trigger_render(input, frame.tex_allocator());
                                }
                            } else {
                                ui.vertical_centered_justified(|ui| {
                                    let button = egui::widgets::Button::new("Re-render image!");
                                    if ui.add(button).clicked() {
                                        self.trigger_render(input, frame.tex_allocator());
                                    }
                                });
                            }
                        }
                        Err(err) => {
                            ui.colored_label(
                                egui::Color32::RED,
                                format!("Error detected:\n{:?}", err),
                            );
                        }
                    }
                    ui.end_row();
                })
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(ref mut data) = self.data {
                let image_sizing = if self.config.display_actual_size {
                    egui::Vec2::new(
                        data.last_render_width as f32,
                        data.last_render_height as f32,
                    )
                } else {
                    ui.available_size()
                };

                egui::ScrollArea::auto_sized().show(ui, |ui| {
                    if let Some(tex_id) = data.last_render_tex {
                        ui.image(tex_id, image_sizing);
                    }
                });
            }
        });
    }
}

fn path_edit_singleline(ui: &mut egui::Ui, path_buf: &mut PathBuf) {
    let mut temp = path_buf.to_string_lossy().to_string();
    ui.text_edit_singleline(&mut temp);
    *path_buf = PathBuf::from(temp);
}

fn vec3_editor(ui: &mut egui::Ui, label: &str, v: &mut Vec3) {
    let speed = 0.01;

    ui.horizontal(|ui| {
        ui.label("x");
        ui.add(egui::widgets::DragValue::new(&mut v.x).speed(speed));
        ui.label("y");
        ui.add(egui::widgets::DragValue::new(&mut v.y).speed(speed));
        ui.label("z");
        ui.add(egui::widgets::DragValue::new(&mut v.z).speed(speed));

        ui.label(label);
    });
}
