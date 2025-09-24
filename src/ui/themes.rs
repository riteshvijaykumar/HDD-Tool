use eframe::egui;

pub struct SecureTheme;

impl SecureTheme {
    pub fn apply(ctx: &egui::Context) {
        let mut visuals = egui::Visuals::dark();
        
        // SHREDX-inspired color scheme
        visuals.window_fill = egui::Color32::from_rgb(15, 23, 42);        // Dark blue background
        visuals.panel_fill = egui::Color32::from_rgb(20, 28, 47);         // Slightly lighter blue for panels
        visuals.extreme_bg_color = egui::Color32::from_rgb(8, 15, 32);    // Very dark blue
        
        // Button colors
        visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(37, 99, 235);     // Blue buttons
        visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(59, 130, 246);     // Lighter blue on hover
        visuals.widgets.active.bg_fill = egui::Color32::from_rgb(29, 78, 216);       // Darker blue when pressed
        
        // Text colors
        visuals.widgets.inactive.fg_stroke.color = egui::Color32::WHITE;
        visuals.widgets.hovered.fg_stroke.color = egui::Color32::WHITE;
        visuals.widgets.active.fg_stroke.color = egui::Color32::WHITE;
        
        // Table/grid colors
        visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(30, 41, 59); // Table rows
        visuals.selection.bg_fill = egui::Color32::from_rgba_premultiplied(37, 99, 235, 100); // Selection color
        
        // Window borders
        visuals.window_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(71, 85, 105));
        
        ctx.set_visuals(visuals);
    }
    
    // Color constants for consistent usage
    pub const PRIMARY_BLUE: egui::Color32 = egui::Color32::from_rgb(37, 99, 235);
    pub const LIGHT_BLUE: egui::Color32 = egui::Color32::from_rgb(59, 130, 246);
    pub const DARK_BLUE: egui::Color32 = egui::Color32::from_rgb(29, 78, 216);
    pub const DANGER_RED: egui::Color32 = egui::Color32::from_rgb(239, 68, 68);
    pub const SUCCESS_GREEN: egui::Color32 = egui::Color32::from_rgb(34, 197, 94);
    pub const TABLE_ROW: egui::Color32 = egui::Color32::from_rgb(30, 41, 59);
    pub const TABLE_ROW_ALT: egui::Color32 = egui::Color32::from_rgb(25, 35, 52);
}