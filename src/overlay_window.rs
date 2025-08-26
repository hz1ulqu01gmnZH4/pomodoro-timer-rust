use egui::{Color32, Pos2, Rect, Vec2, TextureHandle, ViewportBuilder, ViewportId, ViewportCommand};
use rand::Rng;
use std::time::{Duration, Instant};

#[path = "windows_transparency.rs"]
mod windows_transparency;
use windows_transparency::enable_window_transparency;

const TOMATO_SIZE: f32 = 60.0;
const SPAWN_RATE: f32 = 0.1;
const ANIMATION_DURATION: Duration = Duration::from_secs(15);

pub struct Tomato {
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

pub struct TomatoOverlay {
    tomatoes: Vec<Tomato>,
    texture: Option<TextureHandle>,
    start_time: Instant,
    elapsed_time: f32,
    viewport_id: ViewportId,
    active: bool,
    transparency_enabled: bool,
    frame_count: u32,
}

impl TomatoOverlay {
    pub fn new() -> Self {
        Self {
            tomatoes: Vec::new(),
            texture: None,
            start_time: Instant::now(),
            elapsed_time: 0.0,
            viewport_id: ViewportId::from_hash_of("tomato_overlay"),
            active: false,
            transparency_enabled: false,
            frame_count: 0,
        }
    }

    pub fn show(&mut self) {
        self.active = true;
        self.start_time = Instant::now();
        self.elapsed_time = 0.0;
        self.tomatoes.clear();
        self.transparency_enabled = false;
        self.frame_count = 0;
    }

    pub fn update(&mut self, ctx: &egui::Context) {
        if !self.active {
            return;
        }

        // Check if we should close
        if self.start_time.elapsed() > ANIMATION_DURATION {
            self.active = false;
            ctx.send_viewport_cmd_to(self.viewport_id, ViewportCommand::Close);
            return;
        }

        // Show overlay window - use fullscreen without decorations
        ctx.show_viewport_immediate(
            self.viewport_id,
            ViewportBuilder::default()
                .with_title("Tomato Rain")
                .with_fullscreen(true)
                .with_decorations(false)
                .with_transparent(true)
                .with_always_on_top()
                .with_mouse_passthrough(false), // Don't use mouse_passthrough to keep keyboard input
            |ctx, _class| {
                self.update_overlay(ctx);
            },
        );
    }

    fn update_overlay(&mut self, ctx: &egui::Context) {
        ctx.request_repaint();
        self.frame_count += 1;
        
        // Apply Windows transparency after a few frames to ensure window is created
        #[cfg(target_os = "windows")]
        if !self.transparency_enabled && self.frame_count > 2 {
            self.transparency_enabled = true;
            
            // Use Windows API to find our window by title and apply transparency
            unsafe {
                use winapi::um::winuser::{FindWindowW, GetForegroundWindow};
                use std::ffi::OsStr;
                use std::os::windows::ffi::OsStrExt;
                
                // Try to get the foreground window (should be our overlay)
                let hwnd = GetForegroundWindow();
                if !hwnd.is_null() {
                    enable_window_transparency(hwnd as isize);
                } else {
                    // Fallback: try to find by window title
                    let mut window_name: Vec<u16> = OsStr::new("Tomato Rain")
                        .encode_wide()
                        .collect();
                    window_name.push(0); // Add null terminator
                    let hwnd = FindWindowW(std::ptr::null(), window_name.as_ptr());
                    if !hwnd.is_null() {
                        enable_window_transparency(hwnd as isize);
                    }
                }
            }
        }

        // Check for escape key - read input without calling ctx methods inside closure
        let escape_pressed = ctx.input(|i| i.key_pressed(egui::Key::Escape));
        
        if escape_pressed {
            self.active = false;
            // Safe to call now - we're outside the ctx.input lock
            ctx.send_viewport_cmd_to(self.viewport_id, ViewportCommand::Close);
            return;
        }

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

        // CRITICAL: Use a transparent CentralPanel to avoid black background
        egui::CentralPanel::default()
            .frame(egui::Frame::none()) // No background fill
            .show(ctx, |ui| {
                // Draw tomatoes on transparent background
                let painter = ui.painter();
                
                if let Some(texture) = &self.texture {
                    for tomato in &self.tomatoes {
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