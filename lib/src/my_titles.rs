use std::{fmt, fs::File, path::PathBuf, time::Duration};

use egui::{Align2, Image, RichText, Ui};
use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
pub struct MyTitlesApp {
    titles: Vec<Title>,
}

#[derive(Serialize, Deserialize, Debug)]
struct TitleId(pub u32);

impl fmt::Display for TitleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "tt{:07}", self.0)
    }
}

#[derive(Serialize, Deserialize, Debug)]
enum TitleType {
    Movie,
    Series,
}

impl fmt::Display for TitleType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TitleType::Movie => write!(f, "Movie"),
            TitleType::Series => write!(f, "TV Series"),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct Title {
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

// tmdb key: 5dbc48ef39494d480c8be7b5308ee261
// read acc tok: eyJhbGciOiJIUzI1NiJ9.eyJhdWQiOiI1ZGJjNDhlZjM5NDk0ZDQ4MGM4YmU3YjUzMDhlZTI2MSIsInN1YiI6IjY1OTEyYjU3YTU4OTAyNzFlYjk2MTg0OCIsInNjb3BlcyI6WyJhcGlfcmVhZCJdLCJ2ZXJzaW9uIjoxfQ.-NPuNM3r8vSuRVyIWTwzrHiV2WbMOdLh-O8pRBDwyUw

macro_rules! ok_or {
    ($e:expr, $err:expr) => {{
        match $e {
            Ok(r) => r,
            Err(_) => $err,
        }
    }};
}

impl MyTitlesApp {
    fn import_from_path(&mut self, path: PathBuf) {
        let Ok(file) = File::open(path) else {
            return;
        };
        let mut rdr = csv::Reader::from_reader(file);
        log::info!("{:?}", ok_or!(rdr.headers(), return));
        // for record in rdr.records() {
        //     log::info!("{:?}", ok_or!(record, return));
        // }
    }
}
impl eframe::App for MyTitlesApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let titles = [
            Title {
                id: TitleId(816692),
                title: "Interstellar".to_string(),
                ty: TitleType::Movie,
                year: 2014,
                rating: 8.7,
                my_rating: Some(6.0),
                directors: vec!["Cristopher Nolan".to_string()],
                actors: vec![
                    "Matthew McConaughey".to_string(),
                    "Anne Hathaway".to_string(),
                    "Jessica Chastain".to_string(),
                ],
                genres: vec![
                    "Adventure".to_string(),
                    "Drama".to_string(),
                    "Science Fiction".to_string(),
                ],
                poster_img: "/gEU2QniE6E77NI6lCU6MxlNBvIx.jpg".to_string(),
                description: "Changed desc 2 of explorers who make use of a newly discovered wormhole to surpass the limitations on human space travel and conquer the vast distances involved in an interstellar voyage.".to_string(),
            },
            Title {
                id: TitleId(816692),
                title: "Interstellar".to_string(),
                ty: TitleType::Movie,
                year: 2014,
                rating: 8.7,
                my_rating: None,
                directors: vec!["Cristopher Nolan".to_string()],
                actors: vec![
                    "Matthew McConaughey".to_string(),
                    "Anne Hathaway".to_string(),
                    "Jessica Chastain".to_string(),
                ],
                genres: vec![
                    "Adventure".to_string(),
                    "Drama".to_string(),
                    "Science Fiction".to_string(),
                ],
                poster_img: "/gEU2QniE6E77NI6lCU6MxlNBvIx.jpg".to_string(),
                description: "The adventures of a group of explorers who make use of a newly discovered wormhole to surpass the limitations on human space travel and conquer the vast distances involved in an interstellar voyage.".to_string(),
            },
        ];
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Your Titles");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Max), |ui| {
                    if ui.button("Import from IMDB").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            if let Some(ext) = path.extension() {
                                if ext != "csv" {
                                    log::error!("wrong ext");
                                } else {
                                    self.import_from_path(path);
                                }
                            } else {
                                log::error!("wrong ext");
                            }
                        }
                    }
                });
            });
            ui.separator();
            for title in titles {
                ui.horizontal(|ui| {
                    ui.add(
                        Image::new(format!("{}{}", TMDB_IMG_BASE, title.poster_img))
                            .fit_to_original_size(0.2),
                    );
                    ui.vertical(|ui| {
                        ui.style_mut().spacing.item_spacing = egui::Vec2::new(0.0, 7.0);
                        ui.hyperlink_to(
                            RichText::new(title.title).heading(),
                            format!("{}{}", IMDB_TITLE_BASE, title.id),
                        );
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
                        ui.label(title.description);
                    });
                });
                ui.separator();
            }
        });
    }
}
