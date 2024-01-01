use std::{
    collections::BTreeMap,
    fs::File,
    ops::DerefMut,
    path::PathBuf,
    sync::{mpsc::channel, Arc, Mutex},
};

use egui::{Image, RichText};
use serde::{Deserialize, Serialize};

use crate::models::{tmdb, ImdbCsvRow, TitleId, TitleType};
use egui_notify::Toasts;

pub type Titles = BTreeMap<String, Title>;

pub struct MyTitlesApp {
    pub titles: Titles,
    pub input_id: String,
    pub toasts: Arc<Mutex<Toasts>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Title {
    id: TitleId,
    title: String,
    year: u16,
    ty: TitleType,
    rating: f32,
    my_rating: Option<f32>,
    directors: Vec<String>,
    actors: Vec<String>,
    genres: Vec<String>,
    poster_img: String,
    description: String,
}

const TMDB_IMG_BASE: &str = "https://image.tmdb.org/t/p/w500";
const IMDB_TITLE_BASE: &str = "https://www.imdb.com/title/";

macro_rules! ok_or {
    ($e:expr, $err:expr) => {{
        match $e {
            Ok(r) => r,
            Err(_) => $err,
        }
    }};
}

fn make_request<T: serde::de::DeserializeOwned + Send + 'static>(url: String) -> T {
    let req = ehttp::Request {
        url,
        method: "GET".to_owned(),
        body: Vec::new(),
        headers: ehttp::headers(&[
            ("Authorization", "Bearer eyJhbGciOiJIUzI1NiJ9.eyJhdWQiOiI1ZGJjNDhlZjM5NDk0ZDQ4MGM4YmU3YjUzMDhlZTI2MSIsInN1YiI6IjY1OTEyYjU3YTU4OTAyNzFlYjk2MTg0OCIsInNjb3BlcyI6WyJhcGlfcmVhZCJdLCJ2ZXJzaW9uIjoxfQ.-NPuNM3r8vSuRVyIWTwzrHiV2WbMOdLh-O8pRBDwyUw"),
            ("Accept", "application/json")
        ])
    };
    let (tx, rx) = channel();

    ehttp::fetch(req, move |response| {
        let response = response.unwrap();
        let result: T = ok_or!(serde_json::from_slice(&response.bytes), return);
        tx.send(result).unwrap();
    });
    rx.recv().unwrap()
}

fn get_title_id_from_imdb(id: TitleId) -> Result<(i32, TitleType), ()> {
    let result: tmdb::FindById = make_request(format!(
        "https://api.themoviedb.org/3/find/{}?external_source=imdb_id",
        id
    ));
    if result.movie_results.len() == 1 && result.tv_results.len() == 0 {
        Ok((result.movie_results[0].id, TitleType::Movie))
    } else if result.movie_results.len() == 0 && result.tv_results.len() == 1 {
        Ok((result.movie_results[0].id, TitleType::Series))
    } else {
        Err(())
    }
}

fn get_title_from_tmdb(id: i32, typ: TitleType, my_rating: Option<f32>) -> Title {
    let detail_url = match typ {
        TitleType::Movie => format!(
            "https://api.themoviedb.org/3/movie/{id}?append_to_response=credits&language=en-US"
        ),
        TitleType::Series => format!(
            "https://api.themoviedb.org/3/tv/{id}?append_to_response=credits&language=en-US"
        ),
    };
    let details: tmdb::DetailWCredits = make_request(detail_url);
    let directors = match typ {
        TitleType::Movie => details
            .credits
            .crew
            .into_iter()
            .filter_map(|c| {
                if c.job == "Director" {
                    Some(c.name)
                } else {
                    None
                }
            })
            .collect(),
        TitleType::Series => details
            .created_by
            .unwrap()
            .into_iter()
            .map(|c| c.name)
            .collect(),
    };

    let actors = details
        .credits
        .cast
        .iter()
        .take(3)
        .map(|c| c.name.clone())
        .collect();

    let genres = details.genres.iter().map(|g| g.name.clone()).collect();

    Title {
        id: details.imdb_id.parse().unwrap(),
        title: details.title,
        year: details.release_date[..4].parse().unwrap(),
        ty: typ,
        rating: details.rating,
        my_rating,
        directors,
        actors,
        genres,
        poster_img: details.poster_path,
        description: details.overview,
    }
}

impl MyTitlesApp {
    fn add_movie_from_imdb(&mut self, id: TitleId, my_rating: Option<f32>) {
        let (id, typ) = ok_or!(get_title_id_from_imdb(id), {
            self.toasts
                .lock()
                .unwrap()
                .deref_mut()
                .error("Wrong IMDb ID");
            return;
        });

        let title = get_title_from_tmdb(id, typ, my_rating);

        self.titles.insert(title.title.clone(), title);
    }

    fn add_movie_from_input(&mut self) {
        let id = ok_or!(self.input_id.parse(), {
            self.toasts
                .lock()
                .unwrap()
                .deref_mut()
                .error("Wrong IMDb ID");
            return;
        });
        self.add_movie_from_imdb(id, None);
        self.toasts
            .lock()
            .unwrap()
            .deref_mut()
            .success("Added Movie!");
    }
    fn import_from_path(&mut self, path: PathBuf) {
        let Ok(file) = File::open(path) else {
            return;
        };
        let mut rdr = csv::Reader::from_reader(file);
        for record in rdr.deserialize::<ImdbCsvRow>() {
            let record = match record {
                Ok(record) => record,
                Err(_) => {
                    self.toasts
                        .lock()
                        .unwrap()
                        .deref_mut()
                        .error("Wrong file format! Make sure you downloaded from IMDb!");
                    return;
                }
            };
            self.add_movie_from_imdb(record.id, record.rating);
        }
    }
}
impl eframe::App for MyTitlesApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Your Titles");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Max), |ui| {
                    if ui.button("Import").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            if let Some(ext) = path.extension() {
                                if ext != "csv" {
                                    self.toasts
                                        .lock()
                                        .unwrap()
                                        .deref_mut()
                                        .error("Must be a CSV file!");
                                } else {
                                    self.import_from_path(path);
                                }
                            } else {
                                self.toasts
                                    .lock()
                                    .unwrap()
                                    .deref_mut()
                                    .error("Must be a CSV file!");
                            }
                        }
                    }
                });
            });
            ui.separator();
            ui.horizontal_wrapped(|ui| {
                ui.label("Add a movie");
                let response =
                    ui.add(egui::TextEdit::singleline(&mut self.input_id).hint_text("IMDb id"));
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.add_movie_from_input();
                }
            });
            ui.separator();
            let mut to_remove = None;
            egui::ScrollArea::vertical().show(ui, |ui| {
                for (title_name, title) in self.titles.iter() {
                    ui.horizontal_wrapped(|ui| {
                        ui.add(
                            Image::new(format!("{}{}", TMDB_IMG_BASE, title.poster_img))
                                .fit_to_original_size(0.2),
                        );
                        ui.vertical(|ui| {
                            // ui.style_mut().spacing.item_spacing = egui::Vec2::new(0.0, 7.0);
                            ui.horizontal(|ui| {
                                ui.hyperlink_to(
                                    RichText::new(title.title.clone()).heading(),
                                    format!("{}{}", IMDB_TITLE_BASE, title.id),
                                );
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Max),
                                    |ui| {
                                        if ui.button("Delete").clicked() {
                                            to_remove = Some(title_name.clone());
                                        }
                                    },
                                );
                            });
                            ui.horizontal(|ui| {
                                ui.label(title.year.to_string());
                                ui.separator();
                                ui.label(title.ty.to_string());
                                ui.separator();
                                ui.label(title.genres.join(", "));
                            });
                            let my_rating = if let Some(rating) = title.my_rating {
                                format!("★ {}", rating)
                            } else {
                                "☆ Not Rated".to_string()
                            };
                            ui.label(format!("★ {} {}", title.rating, my_rating));
                            ui.horizontal(|ui| {
                                ui.label(title.directors.join(", "));
                                ui.separator();
                                ui.label(title.actors.join(", "));
                            });
                            /* ui.horizontal_wrapped(|ui|  */
                            ui.label(title.description.clone());
                        });
                    });
                    ui.separator();
                }
            });
            if let Some(to_remove) = to_remove {
                self.titles.remove(&to_remove);
            }
        });
    }
}
