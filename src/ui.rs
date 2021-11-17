use eframe::{
    egui::{self, TextureId},
    epi,
};
use glam::Vec3;
use rgb::RGB8;
use strum::IntoEnumIterator;

use crate::{RenderCommand, RenderConfig, RenderResult, RenderScene};

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
pub struct TemplateApp {
    config: RenderConfig,
    data: Option<UiData>,

    render_command_tx: flume::Sender<RenderCommand>,
    render_result_rx: flume::Receiver<RenderResult>,
}

impl TemplateApp {
    pub(crate) fn new(
        render_command_tx: flume::Sender<RenderCommand>,
        render_result_rx: flume::Receiver<RenderResult>,
    ) -> Self {
        TemplateApp {
            config: Default::default(),
            data: Default::default(),
            render_command_tx,
            render_result_rx,
        }
    }

    fn trigger_render(&self) {
        println!(
            "Triggering render of {width}x{height} image (total {count} pixels)",
            width = self.config.width,
            height = self.config.height,
            count = self.config.image_pixel_count(),
        );

        self.render_command_tx
            .send(RenderCommand::Render {
                config: self.config.clone(),
            })
            .ok()
            .expect("render command send should succeed");
    }
}

impl epi::App for TemplateApp {
    fn name(&self) -> &str {
        "Crab TV"
    }

    /// Called once before the first frame.
    fn setup(
        &mut self,
        _ctx: &egui::CtxRef,
        _frame: &mut epi::Frame<'_>,
        _storage: Option<&dyn epi::Storage>,
    ) {
        // Load previous app state (if any).
        if let Some(storage) = _storage {
            self.config = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
        }

        if self.config.validate().is_ok() {
            self.trigger_render();
        }
    }

    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, &self.config);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        loop {
            match self.render_result_rx.try_recv() {
                Ok(RenderResult::Reset {
                    image_height,
                    image_width,
                }) => {
                    assert!(image_width > 0);
                    assert!(image_height > 0);

                    if let Some(ref mut d) = self.data {
                        d.clear_texture(frame.tex_allocator());
                    }
                    self.data = Some(UiData::new(image_width, image_height));
                }
                Ok(RenderResult::FullImage { pixels }) => {
                    let data = self
                        .data
                        .as_mut()
                        .expect("ui data must be present for storing pixels");

                    data.store_image(pixels.as_slice(), frame.tex_allocator());

                    data.save_output_to_file(self.config.output_filename.as_ref());
                }
                Err(flume::TryRecvError::Empty) => break,
                Err(flume::TryRecvError::Disconnected) => {
                    panic!("Rendering thread seems to have exited before UI!")
                }
            };
        }

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
                                ui.radio_value(
                                    &mut self.config.scene,
                                    scene.clone(),
                                    format!("{}", scene.to_string()),
                                );
                            }
                        });
                    });
                    ui.end_row();

                    ui.horizontal(|ui| {
                        ui.label("Save as");
                        ui.text_edit_singleline(&mut self.config.output_filename);
                    });
                    ui.end_row();

                    ui.collapsing("Graphical display options", |ui| {
                        ui.checkbox(
                            &mut self.config.display_actual_size,
                            "Display render at actual 1:1 size",
                        );
                    });

                    ui.collapsing("Rendering options", |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Image filename");
                            ui.text_edit_singleline(&mut self.config.model_filename);
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
                    });

                    ui.checkbox(&mut self.config.auto_rerender, "Re-render on config change");
                    ui.end_row();

                    if let Some(err_msg) = self.config.validate().err() {
                        ui.colored_label(egui::Color32::RED, format!("Error: {}", err_msg));
                    } else {
                        if self.config.auto_rerender {
                            if config_before != self.config {
                                self.trigger_render();
                            }
                        } else {
                            ui.vertical_centered_justified(|ui| {
                                let button = egui::widgets::Button::new("Re-render image!");
                                if ui.add(button).clicked() {
                                    self.trigger_render();
                                }
                            });
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
                    let mut available = ui.available_size();
                    available.y -= 25.0;
                    available
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
