use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

#[cfg(feature = "reload")]
use hot_lib::*;
#[cfg(not(feature = "reload"))]
use lib::*;

#[cfg(feature = "reload")]
#[hot_lib_reloader::hot_module(dylib = "lib")]
mod hot_lib {
    pub use lib::{Anchor, MyTitlesApp, RecommendApp, State, ToStore};

    hot_functions_from_file!("lib/src/lib.rs");

    #[lib_change_subscription]
    pub fn subscribe() -> hot_lib_reloader::LibReloadObserver {}
}

pub struct MeeplsApp {
    state: State,
}

impl MeeplsApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        egui_extras::install_image_loaders(&cc.egui_ctx);
        let toasts = Arc::new(Mutex::new(egui_notify::Toasts::default()));
        let state = State {
            selected_anchor: Anchor::default(),
            my_titles: MyTitlesApp {
                titles: BTreeMap::new(),
                toasts: Arc::clone(&toasts),
            },
            recommend: RecommendApp::default(),
            toasts,
        };

        let mut slf = Self { state };
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            if let Some(stored) = eframe::get_value::<ToStore>(storage, eframe::APP_KEY) {
                slf.state.selected_anchor = stored.selected_anchor;
                slf.state.my_titles.titles = stored.titles;
                slf.state.recommend = stored.recommend;
            }
        }

        slf
    }
}

impl eframe::App for MeeplsApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        let to_store = ToStore {
            selected_anchor: self.state.selected_anchor,
            titles: self.state.my_titles.titles.clone(),
            recommend: self.state.recommend,
        };
        eframe::set_value(storage, eframe::APP_KEY, &to_store);
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        render(&mut self.state, ctx, frame);
    }
}

fn main() -> eframe::Result<()> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([300.0, 220.0]),
        ..Default::default()
    };
    eframe::run_native(
        "meepls",
        native_options,
        Box::new(|cc| {
            #[cfg(feature = "reload")]
            {
                let ctx = cc.egui_ctx.clone();
                std::thread::spawn(move || loop {
                    hot_lib::subscribe().wait_for_reload();
                    ctx.request_repaint();
                });
            }
            Box::new(MeeplsApp::new(cc))
        }),
    )
}
