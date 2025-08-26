use eframe::egui;
use egui::{Color32, Rect, Vec2, RichText};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use notify_rust::Notification;

mod overlay_window;
mod timer;
mod check_transparency;
mod windows_transparency;
mod transparent_overlay;
#[cfg(target_os = "windows")]
mod windows_overlay;
// mod tray;

#[cfg(feature = "bevy-overlay")]
mod bevy_overlay;

use timer::{PomodoroTimer, SessionType};

// Overlay imports removed - using transparent_overlay module

const WINDOW_WIDTH: f32 = 400.0;
const WINDOW_HEIGHT: f32 = 600.0;

pub struct PomodoroApp {
    timer: Arc<Mutex<PomodoroTimer>>,
    show_settings: bool,
    work_duration: u32,
    short_break: u32,
    long_break: u32,
}

impl Default for PomodoroApp {
    fn default() -> Self {
        Self {
            timer: Arc::new(Mutex::new(PomodoroTimer::new())),
            show_settings: false,
            work_duration: 25,
            short_break: 5,
            long_break: 15,
        }
    }
}

impl PomodoroApp {
    fn show_tomato_overlay(&mut self) {
        #[cfg(debug_assertions)]
        println!("Triggering tomato overlay animation...");
        
        // Spawn a separate process to show the overlay
        // This avoids the issue of running two eframe event loops in the same process
        let exe = std::env::current_exe().unwrap();
        let result = std::process::Command::new(exe)
            .arg("--overlay")
            .spawn();
            
        #[cfg(debug_assertions)]
        match result {
            Ok(_) => println!("Overlay process spawned successfully"),
            Err(e) => eprintln!("Failed to spawn overlay process: {}", e),
        }
    }

    fn send_notification(&self, session_type: &SessionType) {
        let message = match session_type {
            SessionType::Work => "Work session completed! Time for a break.",
            SessionType::ShortBreak => "Break finished! Ready to work?",
            SessionType::LongBreak => "Long break finished! Let's get back to it!",
        };

        let _ = Notification::new()
            .summary("Pomodoro Timer")
            .body(message)
            .timeout(5000)
            .show();
    }
}

impl eframe::App for PomodoroApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        // CRITICAL: This enables transparency for the overlay
        // The main window will have its background from CentralPanel
        egui::Rgba::TRANSPARENT.to_array()
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Request repaint for smooth timer updates
        ctx.request_repaint_after(Duration::from_millis(100));
        
        // Update overlay
        // Overlay update removed - handled by separate window

        // Check if timer completed
        {
            let mut timer = self.timer.lock().unwrap();
            if timer.just_completed() {
                let session_type = timer.get_session_type();
                timer.clear_completed_flag();
                drop(timer); // Explicitly drop the lock before calling methods that need &mut self
                self.send_notification(&session_type);
                self.show_tomato_overlay();
            }
        }

        // Main UI
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                
                // Title
                ui.heading(RichText::new("🍅 Pomodoro Timer").size(32.0).color(Color32::from_rgb(255, 99, 71)));
                ui.add_space(30.0);

                // Timer display
                let mut timer = self.timer.lock().unwrap();
                let time_str = timer.get_time_string();
                let session_str = match timer.get_session_type() {
                    SessionType::Work => "Work Session",
                    SessionType::ShortBreak => "Short Break",
                    SessionType::LongBreak => "Long Break",
                };
                drop(timer);

                ui.label(RichText::new(time_str).size(64.0).strong());
                ui.label(RichText::new(session_str).size(24.0));
                ui.add_space(20.0);

                // Progress bar
                let mut timer = self.timer.lock().unwrap();
                let progress = timer.get_progress();
                drop(timer);
                
                let available_width = ui.available_width() - 40.0;
                let progress_rect = ui.allocate_space(Vec2::new(available_width, 20.0)).1;
                ui.painter().rect_filled(
                    progress_rect,
                    5.0,
                    Color32::from_gray(50),
                );
                let progress_width = available_width * progress;
                let progress_filled_rect = Rect::from_min_size(
                    progress_rect.min,
                    Vec2::new(progress_width, 20.0),
                );
                ui.painter().rect_filled(
                    progress_filled_rect,
                    5.0,
                    Color32::from_rgb(255, 99, 71),
                );
                ui.add_space(30.0);

                // Control buttons
                ui.horizontal(|ui| {
                    let timer = self.timer.lock().unwrap();
                    let is_running = timer.is_running();
                    drop(timer);

                    if !is_running {
                        if ui.button(RichText::new("▶ Start").size(20.0)).clicked() {
                            self.timer.lock().unwrap().start();
                        }
                    } else {
                        if ui.button(RichText::new("⏸ Pause").size(20.0)).clicked() {
                            self.timer.lock().unwrap().pause();
                        }
                    }

                    if ui.button(RichText::new("⏹ Reset").size(20.0)).clicked() {
                        self.timer.lock().unwrap().reset();
                    }

                    if ui.button(RichText::new("⏭ Skip").size(20.0)).clicked() {
                        self.timer.lock().unwrap().skip();
                    }
                });

                ui.add_space(20.0);

                // Session info
                let timer = self.timer.lock().unwrap();
                let session_count = timer.get_session_count();
                drop(timer);
                ui.label(format!("Session {} of 4", session_count));

                ui.add_space(40.0);

                // Settings toggle
                if ui.button(RichText::new("⚙ Settings").size(18.0)).clicked() {
                    self.show_settings = !self.show_settings;
                }

                // Settings panel
                if self.show_settings {
                    ui.add_space(20.0);
                    ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.label(RichText::new("Timer Settings").size(20.0).strong());
                            ui.add_space(10.0);

                            ui.horizontal(|ui| {
                                ui.label("Work Duration (min):");
                                ui.add(egui::Slider::new(&mut self.work_duration, 1..=60));
                            });

                            ui.horizontal(|ui| {
                                ui.label("Short Break (min):");
                                ui.add(egui::Slider::new(&mut self.short_break, 1..=30));
                            });

                            ui.horizontal(|ui| {
                                ui.label("Long Break (min):");
                                ui.add(egui::Slider::new(&mut self.long_break, 1..=60));
                            });

                            if ui.button("Apply Settings").clicked() {
                                let mut timer = self.timer.lock().unwrap();
                                timer.update_durations(
                                    self.work_duration,
                                    self.short_break,
                                    self.long_break,
                                );
                            }
                        });
                    });
                }
            });
        });
    }
}

pub fn run() -> Result<(), eframe::Error> {
    run_app()
}

fn main() -> Result<(), eframe::Error> {
    // Check if we're being launched as an overlay process
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "--overlay" {
        // Run the overlay and exit
        transparent_overlay::TransparentOverlay::show();
        return Ok(());
    }
    
    // On Windows, allocate a console for debugging only in debug builds
    #[cfg(all(windows, debug_assertions))]
    {
        unsafe {
            // Try to allocate console for debug output
            winapi::um::consoleapi::AllocConsole();
            println!("Debug console initialized");
        }
    }
    
    let result = run();
    
    // If there was an error in debug mode, keep console open on Windows
    #[cfg(all(windows, debug_assertions))]
    if result.is_err() {
        println!("Press Enter to exit...");
        let mut input = String::new();
        let _ = std::io::stdin().read_line(&mut input);
    }
    
    result
}

fn run_app() -> Result<(), eframe::Error> {
    // Check transparency support
    if !check_transparency::check_transparency_support() {
        println!("WARNING: Transparency may not be supported");
        check_transparency::get_gpu_info();
    }
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT])
            .with_resizable(false)
            .with_icon(load_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "Rust Pomodoro Timer",
        options,
        Box::new(|_cc| Box::new(PomodoroApp::default())),
    )
}

fn load_icon() -> Arc<egui::IconData> {
    let icon_bytes = include_bytes!("../assets/tomato.png");
    match image::load_from_memory(icon_bytes) {
        Ok(image) => {
            let rgba = image.to_rgba8();
            let (width, height) = rgba.dimensions();
            Arc::new(egui::IconData {
                rgba: rgba.into_raw(),
                width,
                height,
            })
        }
        Err(e) => {
            eprintln!("Failed to load icon: {}. Using default.", e);
            // Return a simple 32x32 red square as fallback
            let size = 32;
            let red_square = vec![255, 0, 0, 255].repeat((size * size) as usize);
            Arc::new(egui::IconData {
                rgba: red_square,
                width: size,
                height: size,
            })
        }
    }
}
