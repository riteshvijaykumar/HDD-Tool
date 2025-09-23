use eframe::egui;

pub struct ProgressWidget {
    pub progress: f32,
    pub status: String,
}

impl ProgressWidget {
    pub fn new() -> Self {
        Self {
            progress: 0.0,
            status: "Ready".to_string(),
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.label(&self.status);
        ui.add(egui::ProgressBar::new(self.progress).show_percentage());
    }
}