use bevy::prelude::*;
use bevy::window::{CursorOptions, WindowLevel, WindowMode};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Receiver, Sender};

const TOMATO_SIZE: f32 = 60.0;
const FALL_SPEED: f32 = 200.0;
const SPAWN_RATE: f32 = 0.1;
const ANIMATION_DURATION: f32 = 15.0;

#[derive(Component)]
struct Tomato {
    velocity: f32,
    sway_amount: f32,
    sway_speed: f32,
    sway_offset: f32,
    original_x: f32,
}

#[derive(Resource)]
struct OverlayState {
    start_time: f32,
    should_close: Arc<Mutex<bool>>,
    close_sender: Sender<()>,
}

#[derive(Resource)]
struct TomatoTexture(Handle<Image>);

pub struct BevyOverlay {
    should_close: Arc<Mutex<bool>>,
    close_receiver: Option<Receiver<()>>,
}

impl BevyOverlay {
    pub fn new() -> Self {
        Self {
            should_close: Arc::new(Mutex::new(false)),
            close_receiver: None,
        }
    }

    pub fn show(&mut self) {
        let should_close = Arc::clone(&self.should_close);
        *should_close.lock().unwrap() = false;
        
        let (tx, rx) = channel();
        self.close_receiver = Some(rx);
        
        let should_close_clone = Arc::clone(&self.should_close);
        
        // Run Bevy in a separate thread
        std::thread::spawn(move || {
            App::new()
                .add_plugins(DefaultPlugins.set(WindowPlugin {
                    primary_window: Some(Window {
                        mode: WindowMode::BorderlessFullscreen(bevy::window::MonitorSelection::Current),
                        transparent: true,
                        decorations: false,
                        window_level: WindowLevel::AlwaysOnTop,
                        cursor_options: CursorOptions { 
                            hit_test: false,
                            ..default()
                        },
                        ..default()
                    }),
                    ..default()
                }))
                .insert_resource(ClearColor(Color::NONE))
                .insert_resource(OverlayState {
                    start_time: 0.0,
                    should_close: should_close_clone,
                    close_sender: tx,
                })
                .add_systems(Startup, setup)
                .add_systems(Update, (
                    spawn_tomatoes,
                    update_tomatoes,
                    check_timer,
                    handle_escape,
                ))
                .run();
        });
    }

    pub fn is_active(&self) -> bool {
        if let Some(rx) = &self.close_receiver {
            // Check if closed without blocking
            match rx.try_recv() {
                Ok(_) => false,
                Err(_) => true,
            }
        } else {
            false
        }
    }
    
    pub fn update(&mut self, _ctx: &egui::Context) {
        // Bevy handles its own update loop, so this is a no-op
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut state: ResMut<OverlayState>,
    time: Res<Time>,
) {
    // Camera
    commands.spawn(Camera2d);
    
    // Load tomato texture
    let texture = asset_server.load("tomato.png");
    commands.insert_resource(TomatoTexture(texture));
    
    // Initialize start time
    state.start_time = time.elapsed_secs();
}

fn spawn_tomatoes(
    mut commands: Commands,
    windows: Query<&Window>,
    texture: Res<TomatoTexture>,
    time: Res<Time>,
) {
    let window = windows.single();
    let width = window.resolution.width();
    
    // Spawn tomatoes randomly
    if rand::random::<f32>() < SPAWN_RATE * time.delta_secs() {
        let x = rand::random::<f32>() * width - width / 2.0;
        
        commands.spawn((
            (
                Sprite::from_image(texture.0.clone()),
                Transform::from_xyz(x, window.resolution.height() / 2.0 + TOMATO_SIZE, 0.0)
                    .with_scale(Vec3::splat(TOMATO_SIZE / 128.0)), // Assuming original is 128x128
            ),
            Tomato {
                velocity: rand::random::<f32>() * 200.0 + 150.0,
                sway_amount: rand::random::<f32>() * 30.0 + 20.0,
                sway_speed: rand::random::<f32>() * 2.0 + 2.0,
                sway_offset: rand::random::<f32>() * std::f32::consts::TAU,
                original_x: x,
            },
        ));
    }
}

fn update_tomatoes(
    mut commands: Commands,
    mut tomatoes: Query<(Entity, &mut Transform, &Tomato)>,
    windows: Query<&Window>,
    time: Res<Time>,
) {
    let window = windows.single();
    let height = window.resolution.height();
    
    for (entity, mut transform, tomato) in &mut tomatoes {
        // Update position
        transform.translation.y -= tomato.velocity * time.delta_secs();
        
        // Sway motion
        let sway = (time.elapsed_secs() * tomato.sway_speed + tomato.sway_offset).sin() * tomato.sway_amount;
        transform.translation.x = tomato.original_x + sway;
        
        // Remove if off screen
        if transform.translation.y < -height / 2.0 - TOMATO_SIZE {
            commands.entity(entity).despawn();
        }
    }
}

fn check_timer(
    state: Res<OverlayState>,
    time: Res<Time>,
    mut exit: EventWriter<AppExit>,
) {
    // Check if should close from external signal
    if *state.should_close.lock().unwrap() {
        exit.send(AppExit::Success);
        return;
    }
    
    // Check if timer expired
    if time.elapsed_secs() - state.start_time > ANIMATION_DURATION {
        exit.send(AppExit::Success);
    }
}

fn handle_escape(
    keys: Res<ButtonInput<KeyCode>>,
    mut exit: EventWriter<AppExit>,
    state: Res<OverlayState>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        let _ = state.close_sender.send(());
        exit.send(AppExit::Success);
    }
}