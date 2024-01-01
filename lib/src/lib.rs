mod models;
mod my_titles;
mod recommend;

use std::{
    ops::DerefMut,
    sync::{Arc, Mutex},
};

pub use my_titles::MyTitlesApp;
use my_titles::Titles;
pub use recommend::RecommendApp;

#[derive(Copy, Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum Anchor {
    MyTitles,
    Recommend,
}

impl std::fmt::Display for Anchor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl From<Anchor> for egui::WidgetText {
    fn from(value: Anchor) -> Self {
        Self::RichText(egui::RichText::new(value.to_string()))
    }
}

impl Default for Anchor {
    fn default() -> Self {
        Self::MyTitles
    }
}
#[derive(Default, Debug, serde::Deserialize, serde::Serialize)]
pub struct ToStore {
    pub selected_anchor: Anchor,
    pub titles: Titles,
    pub recommend: RecommendApp,
}

pub struct State {
    pub selected_anchor: Anchor,
    pub my_titles: MyTitlesApp,
    pub recommend: RecommendApp,
    pub toasts: Arc<Mutex<egui_notify::Toasts>>,
}

impl State {
    fn bar_contents(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let mut selected_anchor = self.selected_anchor;
        for (name, anchor, _app) in self.apps_iter_mut() {
            if ui
                .selectable_label(selected_anchor == anchor, name)
                .clicked()
            {
                selected_anchor = anchor;
            }
        }
        self.selected_anchor = selected_anchor;
        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            egui::warn_if_debug_build(ui);
        });
    }

    fn apps_iter_mut(&mut self) -> impl Iterator<Item = (&str, Anchor, &mut dyn eframe::App)> {
        let vec = vec![
            (
                "My Titles",
                Anchor::MyTitles,
                &mut self.my_titles as &mut dyn eframe::App,
            ),
            (
                "Recommend",
                Anchor::Recommend,
                &mut self.recommend as &mut dyn eframe::App,
            ),
        ];
        vec.into_iter()
    }

    fn show_selected_app(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let selected_anchor = self.selected_anchor;
        for (_name, anchor, app) in self.apps_iter_mut() {
            if anchor == selected_anchor || ctx.memory(|mem| mem.everything_is_visible()) {
                app.update(ctx, frame);
            }
        }
    }
}

#[no_mangle]
pub fn render(state: &mut State, ctx: &egui::Context, frame: &mut eframe::Frame) {
    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        // The top panel is often a good place for a menu bar:

        egui::menu::bar(ui, |ui| {
            // NOTE: no File->Quit on web pages!
            let is_web = cfg!(target_arch = "wasm32");
            if !is_web {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.add_space(16.0);
            }

            egui::widgets::global_dark_light_mode_buttons(ui);
        });
    });

    egui::SidePanel::left("nav").show(ctx, |ui| {
        ui.vertical(|ui| {
            ui.visuals_mut().button_frame = false;
            state.bar_contents(ui, frame)
        });
    });
    egui::TopBottomPanel::bottom("powered").show(ctx, |ui| {
        powered_by_egui_and_eframe(ui);
    });
    state.show_selected_app(ctx, frame);
    state.toasts.lock().unwrap().deref_mut().show(ctx);
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
