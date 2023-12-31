#[derive(Default, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct RecommendApp {}

impl eframe::App for RecommendApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Recommend");
        });
    }
}
