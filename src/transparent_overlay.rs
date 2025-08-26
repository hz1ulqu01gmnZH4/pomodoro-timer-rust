use eframe::{egui, NativeOptions};
use egui::{Color32, Pos2, Vec2};
use std::time::{Duration, Instant};
use rand::Rng;
use raw_window_handle::HasRawWindowHandle;

#[cfg(target_os = "windows")]
use crate::windows_overlay;

const TOMATO_SIZE: f32 = 60.0;
const SPAWN_RATE: f32 = 0.1;
const ANIMATION_DURATION: Duration = Duration::from_secs(15);

#[derive(Clone)]
struct Tomato {
    pos: Pos2,
    velocity: f32,
    rotation: f32,
    rotation_speed: f32,
    sway_amount: f32,
    sway_speed: f32,
    sway_offset: f32,
    original_x: f32,
}

impl Tomato {
    fn new(window_width: f32) -> Self {
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(0.0..window_width);
        
        Self {
            pos: Pos2::new(x, -TOMATO_SIZE),
            velocity: rng.gen_range(150.0..350.0),
            rotation: rng.gen_range(0.0..std::f32::consts::TAU),
            rotation_speed: rng.gen_range(-2.0..2.0),
            sway_amount: rng.gen_range(20.0..50.0),
            sway_speed: rng.gen_range(2.0..4.0),
            sway_offset: rng.gen_range(0.0..std::f32::consts::TAU),
            original_x: x,
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

pub struct TransparentOverlay {
    tomatoes: Vec<Tomato>,
    texture: Option<egui::TextureHandle>,
    start_time: Instant,
    elapsed_time: f32,
    spawn_timer: f32,
    active: bool,
    #[cfg(target_os = "windows")]
    hwnd_installed: bool,
    esc_was_down: bool,
}

impl TransparentOverlay {
    pub fn new() -> Self {
        Self {
            tomatoes: Vec::new(),
            texture: None,
            start_time: Instant::now(),
            elapsed_time: 0.0,
            spawn_timer: 0.0,
            active: true,
            #[cfg(target_os = "windows")]
            hwnd_installed: false,
            esc_was_down: false,
        }
    }

    pub fn show() {
        #[cfg(debug_assertions)]
        println!("Starting tomato overlay animation...");
        
        let options = NativeOptions {
            renderer: eframe::Renderer::Glow,  // Use Glow for better transparency on Windows
            viewport: egui::ViewportBuilder::default()
                .with_decorations(false)
                .with_transparent(true)
                .with_always_on_top()
                .with_mouse_passthrough(true)  // Make window click-through
                .with_resizable(false),
            ..Default::default()
        };

        match eframe::run_native(
            "Tomato Overlay",
            options,
            Box::new(|cc| {
                #[cfg(debug_assertions)]
                println!("Overlay window created");
                
                // Get the monitor size and position the window to cover the screen
                let viewport_id = cc.egui_ctx.viewport_id();
                // Try to maximize the window to cover the screen
                cc.egui_ctx.send_viewport_cmd_to(
                    viewport_id,
                    egui::ViewportCommand::Maximized(true)
                );
                
                // Note: Windows-specific transparency will be applied in the first update() call
                // when we can get the window handle
                
                Box::new(TransparentOverlay::new())
            }),
        ) {
            #[cfg(debug_assertions)]
            Ok(_) => println!("Overlay finished successfully"),
            #[cfg(debug_assertions)]
            Err(e) => eprintln!("Overlay error: {}", e),
            #[cfg(not(debug_assertions))]
            _ => {},
        }
    }

    fn update_overlay(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        let window_size = ctx.screen_rect().size();
        let dt = ctx.input(|i| i.stable_dt);
        self.elapsed_time = self.start_time.elapsed().as_secs_f32();

        // Load texture if not loaded
        if self.texture.is_none() {
            // Try multiple possible paths for the tomato image
            let possible_paths = [
                "assets/tomato.png",
                "rust-pomodoro/assets/tomato.png",
                "../assets/tomato.png",
            ];
            
            for path in &possible_paths {
                if let Ok(image_data) = std::fs::read(path) {
                    if let Ok(image) = image::load_from_memory(&image_data) {
                        let size = [image.width() as _, image.height() as _];
                        let image_buffer = image.to_rgba8();
                        let pixels = image_buffer.as_flat_samples();
                        
                        let color_image = egui::ColorImage::from_rgba_unmultiplied(
                            size,
                            pixels.as_slice(),
                        );
                        
                        self.texture = Some(ctx.load_texture(
                            "tomato",
                            color_image,
                            Default::default(),
                        ));
                        println!("Successfully loaded tomato texture from {}", path);
                        break;
                    }
                }
            }
            
            // If still no texture, create a red circle as fallback
            if self.texture.is_none() {
                println!("Warning: Could not load tomato.png, using fallback");
                let size = [60, 60];
                let mut pixels = Vec::new();
                for y in 0..size[1] {
                    for x in 0..size[0] {
                        let dx = x as f32 - 30.0;
                        let dy = y as f32 - 30.0;
                        let dist = (dx * dx + dy * dy).sqrt();
                        if dist < 28.0 {
                            pixels.push(255); // R
                            pixels.push(99);  // G
                            pixels.push(71);  // B
                            pixels.push(255); // A
                        } else {
                            pixels.push(0);
                            pixels.push(0);
                            pixels.push(0);
                            pixels.push(0);
                        }
                    }
                }
                
                let color_image = egui::ColorImage::from_rgba_unmultiplied(
                    size,
                    &pixels,
                );
                
                self.texture = Some(ctx.load_texture(
                    "tomato_fallback",
                    color_image,
                    Default::default(),
                ));
            }
        }

        // Spawn new tomatoes
        self.spawn_timer += dt;
        if self.spawn_timer > SPAWN_RATE {
            self.spawn_timer = 0.0;
            self.tomatoes.push(Tomato::new(window_size.x));
        }

        // Update tomatoes
        self.tomatoes.retain_mut(|tomato| {
            tomato.update(dt, self.elapsed_time);
            !tomato.is_off_screen(window_size.y)
        });

        // Draw tomatoes
        if let Some(texture) = &self.texture {
            let painter = ui.painter();
            
            for tomato in &self.tomatoes {
                painter.add(egui::Shape::image(
                    texture.id(),
                    egui::Rect::from_center_size(
                        tomato.pos,
                        Vec2::splat(TOMATO_SIZE),
                    ),
                    egui::Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                ));
            }
        }

        // Close after animation duration
        if self.elapsed_time > ANIMATION_DURATION.as_secs_f32() {
            self.active = false;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}

impl eframe::App for TransparentOverlay {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        // CRITICAL: This enables transparency for the overlay
        egui::Rgba::TRANSPARENT.to_array()
    }
    
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // One-time Windows setup: install non-activating transparent behavior
        #[cfg(target_os = "windows")]
        if !self.hwnd_installed {
            // Try to get the window handle through raw-window-handle
            use raw_window_handle::RawWindowHandle;
            let raw_handle = frame.raw_window_handle();
            if let RawWindowHandle::Win32(h) = raw_handle {
                // h.hwnd is a *mut c_void, convert directly to isize
                let hwnd_ptr = h.hwnd as isize;
                if hwnd_ptr != 0 {
                    let hwnd = windows::Win32::Foundation::HWND(hwnd_ptr);
                    unsafe {
                        windows_overlay::install_overlay_window(hwnd, None);
                    }
                    self.hwnd_installed = true;
                }
            }
        }
        
        // ESC key handling without requiring focus
        #[cfg(target_os = "windows")]
        {
            let esc_down = windows_overlay::esc_is_down();
            if esc_down && !self.esc_was_down {
                self.active = false;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            self.esc_was_down = esc_down;
            
            if windows_overlay::take_close_requested() {
                self.active = false;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
        
        // Non-Windows ESC handling through normal input
        #[cfg(not(target_os = "windows"))]
        {
            ctx.input(|i| {
                if i.key_pressed(egui::Key::Escape) {
                    self.active = false;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
        }
        
        if !self.active {
            return;
        }

        // Request continuous repaints for smooth animation
        ctx.request_repaint();

        // Clear with transparent background
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(Color32::TRANSPARENT))
            .show(ctx, |ui| {
                self.update_overlay(ctx, ui);
            });
    }
}