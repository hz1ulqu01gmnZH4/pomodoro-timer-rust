// Module to select between egui and bevy overlay implementations

#[cfg(feature = "bevy-overlay")]
pub use crate::bevy_overlay::BevyOverlay as OverlayImpl;

#[cfg(not(feature = "bevy-overlay"))]
pub use crate::overlay_window::TomatoOverlay as OverlayImpl;

pub fn create_overlay() -> Box<dyn OverlayTrait> {
    #[cfg(feature = "bevy-overlay")]
    {
        Box::new(BevyOverlay::new())
    }
    
    #[cfg(not(feature = "bevy-overlay"))]
    {
        Box::new(TomatoOverlay::new())
    }
}

pub trait OverlayTrait {
    fn show(&mut self);
    fn update(&mut self, ctx: &egui::Context);
}