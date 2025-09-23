use eframe::egui;

pub struct SecureTheme;

impl SecureTheme {
    pub fn apply(ctx: &egui::Context) {
        let mut visuals = egui::Visuals::dark();
        visuals.window_fill = egui::Color32::from_rgb(20, 20, 30);
        visuals.panel_fill = egui::Color32::from_rgb(25, 25, 35);
        ctx.set_visuals(visuals);
    }
}