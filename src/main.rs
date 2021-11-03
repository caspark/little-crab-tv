#![deny(clippy::all)] // make all clippy warnings into errors
#![allow(clippy::many_single_char_names)]

mod ui;

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

#[derive(Clone, Debug)]
struct Image {
    width: usize,
    height: usize,
    pixels: Vec<RGB8>,
}

impl Image {
    fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![RGB8::default(); width * height],
        }
    }

    fn pixel(&mut self, x: i32, y: i32) -> &mut RGB8 {
        &mut self.pixels[y as usize * self.width + x as usize]
    }

    // incorrect because it depends on choosing the correct "increment", which will vary based on
    // how many pixels need to be drawn
    #[allow(dead_code)]
    fn line_naive1(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: RGB8) {
        let increment = 0.1;
        for i in 0..((1.0 / increment) as i32) {
            let i = f64::from(i) * increment;
            let x = x0 as f64 + (x1 - x0) as f64 * i;
            let y = y0 as f64 + (y1 - y0) as f64 * i;
            *self.pixel(x as i32, y as i32) = color;
        }
    }

    // incorrect because it doesn't handle the case where the line is near vertical or x1 < x0
    #[allow(dead_code)]
    fn line_naive2(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: RGB8) {
        for x in x0..x1 {
            let t = (x - x0) as f64 / (x1 - x0) as f64;
            let y = y0 as f64 * (1.0 - t) as f64 + y1 as f64 * t as f64;
            *self.pixel(x as i32, y as i32) = color;
        }
    }

    // Bresenham's algorithm 1 - correct but slow due to needing floating point maths
    fn line(&mut self, mut x0: i32, mut y0: i32, mut x1: i32, mut y1: i32, color: RGB8) {
        let steep = if (x0 - x1).abs() < (y0 - y1).abs() {
            std::mem::swap(&mut x0, &mut y0);
            std::mem::swap(&mut x1, &mut y1);
            true
        } else {
            false
        };
        if x0 > x1 {
            std::mem::swap(&mut x0, &mut x1);
            std::mem::swap(&mut y0, &mut y1);
        }

        for x in x0..x1 {
            let t = (x - x0) as f64 / (x1 - x0) as f64;
            let y = y0 as f64 * (1.0 - t) as f64 + y1 as f64 * t as f64;
            if steep {
                *self.pixel(y as i32, x as i32) = color;
            } else {
                *self.pixel(x as i32, y as i32) = color;
            }
        }
    }
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

                let mut image = Image::new(config.image_width, config.image_height);

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
                        pixels: image.pixels,
                    })
                    .ok()
                    .expect("Sending final image shoud succeed");
            }
        }
    }
}
