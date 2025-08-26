use eframe::egui;
use egui::{Color32, Pos2, Rect, Vec2, TextureHandle};
use rand::Rng;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const TOMATO_SIZE: f32 = 60.0;
const SPAWN_RATE: f32 = 0.1; // Probability of spawning a tomato each frame
const ANIMATION_DURATION: Duration = Duration::from_secs(15); // How long the overlay shows

struct Tomato {
    pos: Pos2,
    velocity: f32,
    rotation: f32,
    rotation_speed: f32,
    sway_amount: f32,
    sway_speed: f32,
    original_x: f32,
    sway_offset: f32,
}

impl Tomato {
    fn new(window_width: f32) -> Self {
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(0.0..window_width);
        
        Self {
            pos: Pos2::new(x, -TOMATO_SIZE),
            velocity: rng.gen_range(150.0..350.0),
            rotation: 0.0,
            rotation_speed: rng.gen_range(-3.0..3.0),
            sway_amount: rng.gen_range(20.0..50.0),
            sway_speed: rng.gen_range(2.0..4.0),
            original_x: x,
            sway_offset: rng.gen_range(0.0..std::f32::consts::TAU),
        }
    }

    fn update(&mut self, dt: f32, elapsed: f32) {
        self.pos.y += self.velocity * dt;
        self.rotation += self.rotation_speed * dt;
        
        // Sway motion
        let sway = (elapsed * self.sway_speed + self.sway_offset).sin() * self.sway_amount;
        self.pos.x = self.original_x + sway;
    }

    fn is_off_screen(&self, window_height: f32) -> bool {
        self.pos.y > window_height + TOMATO_SIZE
    }
}

struct OverlayApp {
    tomatoes: Vec<Tomato>,
    texture: Option<TextureHandle>,
    start_time: Instant,
    should_close: Arc<Mutex<bool>>,
    elapsed_time: f32,
}

impl OverlayApp {
    fn new(should_close: Arc<Mutex<bool>>) -> Self {
        Self {
            tomatoes: Vec::new(),
            texture: None,
            start_time: Instant::now(),
            should_close,
            elapsed_time: 0.0,
        }
    }
}

impl eframe::App for OverlayApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check if we should close
        if *self.should_close.lock().unwrap() || self.start_time.elapsed() > ANIMATION_DURATION {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        ctx.request_repaint();

        // Load texture on first frame
        if self.texture.is_none() {
            let image_bytes = include_bytes!("../assets/tomato.png");
            match image::load_from_memory(image_bytes) {
                Ok(image) => {
                    let size = [image.width() as _, image.height() as _];
                    let rgba = image.to_rgba8();
                    let pixels = rgba.as_flat_samples();
                    
                    self.texture = Some(ctx.load_texture(
                        "tomato",
                        egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice()),
                        Default::default(),
                    ));
                }
                Err(e) => {
                    eprintln!("Failed to load tomato texture: {}", e);
                    // Create a simple red circle as fallback
                    let size = 60;
                    let mut pixels = vec![0u8; (size * size * 4) as usize];
                    let center = size as f32 / 2.0;
                    let radius = center - 2.0;
                    
                    for y in 0..size {
                        for x in 0..size {
                            let dx = x as f32 - center;
                            let dy = y as f32 - center;
                            let dist = (dx * dx + dy * dy).sqrt();
                            
                            let idx = ((y * size + x) * 4) as usize;
                            if dist <= radius {
                                pixels[idx] = 255;     // R
                                pixels[idx + 1] = 99;  // G
                                pixels[idx + 2] = 71;  // B
                                pixels[idx + 3] = 255; // A
                            }
                        }
                    }
                    
                    self.texture = Some(ctx.load_texture(
                        "tomato_fallback",
                        egui::ColorImage::from_rgba_unmultiplied([size as _, size as _], &pixels),
                        Default::default(),
                    ));
                }
            }
        }

        let window_size = ctx.screen_rect().size();
        let dt = ctx.input(|i| i.unstable_dt);
        self.elapsed_time += dt;

        // Spawn new tomatoes
        let mut rng = rand::thread_rng();
        if rng.gen::<f32>() < SPAWN_RATE {
            self.tomatoes.push(Tomato::new(window_size.x));
        }

        // Update tomatoes
        for tomato in &mut self.tomatoes {
            tomato.update(dt, self.elapsed_time);
        }

        // Remove off-screen tomatoes
        self.tomatoes.retain(|t| !t.is_off_screen(window_size.y));

        // Draw
        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(ctx, |ui| {
                let painter = ui.painter();
                
                if let Some(texture) = &self.texture {
                    for tomato in &self.tomatoes {
                        // Note: egui 0.24 doesn't support rotation on images directly
                        // We'll just draw without rotation for now
                        painter.add(egui::Shape::image(
                            texture.id(),
                            Rect::from_center_size(
                                tomato.pos,
                                Vec2::splat(TOMATO_SIZE),
                            ),
                            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                            Color32::WHITE,
                        ));
                    }
                }
            });
    }
}

pub fn show_overlay(should_close: Arc<Mutex<bool>>) {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_fullscreen(true)
            .with_decorations(false)
            .with_transparent(true)
            .with_always_on_top()
            .with_mouse_passthrough(true),
        ..Default::default()
    };

    let _ = eframe::run_native(
        "Tomato Rain",
        options,
        Box::new(|_cc| Box::new(OverlayApp::new(should_close))),
    );
}
