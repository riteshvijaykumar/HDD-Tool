use eframe::egui;

pub fn show_logo(ui: &mut egui::Ui) -> egui::Response {
    let desired_size = egui::vec2(120.0, 40.0); // Wider to fit "SHRED" + "X"
    
    ui.allocate_ui(desired_size, |ui| {
        ui.horizontal(|ui| {
            // "SHRED" in bold white
            ui.label(
                egui::RichText::new("SHRED")
                    .size(24.0)
                    .color(egui::Color32::WHITE)
                    .strong() // Bold
            );
            
            // "X" in bold blue  
            ui.label(
                egui::RichText::new("X")
                    .size(24.0)
                    .color(egui::Color32::from_rgb(0, 150, 255)) // Bright blue
                    .strong() // Bold
            );
        });
    }).response
}