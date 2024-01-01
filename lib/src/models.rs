use serde::{de, Deserialize, Deserializer, Serialize};
use std::fmt;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct TitleId(pub u32);

impl fmt::Display for TitleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "tt{:07}", self.0)
    }
}

#[derive(Error, Debug)]
#[error("couldn't parse title id")]
pub struct ParseTitleIdError;

impl FromStr for TitleId {
    type Err = ParseTitleIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < 3 {
            return Err(ParseTitleIdError);
        }
        Ok(TitleId(s[2..].parse().map_err(|_| ParseTitleIdError)?))
    }
}

impl Serialize for TitleId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for TitleId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(de::Error::custom)
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum TitleType {
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

#[derive(Debug, Copy, Clone)]
pub enum CsvTitleType {
    Movie,
    Series,
}

#[derive(Error, Debug)]
#[error("couldn't parse title")]
pub struct ParseTitleError;

impl FromStr for CsvTitleType {
    type Err = ParseTitleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "movie" => Ok(CsvTitleType::Movie),
            "tvSeries" => Ok(CsvTitleType::Series),
            _ => Err(ParseTitleError),
        }
    }
}

impl From<CsvTitleType> for TitleType {
    fn from(value: CsvTitleType) -> Self {
        match value {
            CsvTitleType::Movie => TitleType::Movie,
            CsvTitleType::Series => TitleType::Series,
        }
    }
}

impl<'de> Deserialize<'de> for CsvTitleType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(de::Error::custom)
    }
}

#[derive(Deserialize, Debug)]
pub struct ImdbCsvRow {
    #[serde(rename = "Const")]
    pub id: TitleId,
    #[serde(rename = "Your Rating")]
    pub rating: Option<f32>,
}

pub mod tmdb {
    use serde::Deserialize;

    #[derive(Deserialize, Debug)]
    pub struct TitleFindResult {
        pub id: i32,
    }

    #[derive(Deserialize, Debug)]
    pub struct FindById {
        pub movie_results: Vec<TitleFindResult>,
        pub tv_results: Vec<TitleFindResult>,
    }

    #[derive(Deserialize, Debug)]
    pub struct DetailWCredits {
        #[serde(alias = "name")]
        pub title: String,
        #[serde(alias = "first_air_date")]
        pub release_date: String,
        #[serde(rename = "vote_average")]
        pub rating: f32,
        pub imdb_id: String,
        pub credits: Credits,
        pub genres: Vec<Genre>,
        pub poster_path: String,
        pub overview: String,
        pub created_by: Option<Vec<Cast>>,
    }

    // #[derive(Deserialize, Debug)]
    // pub struct SeriesDetailResponse {
    //     pub name: String,
    //     pub first_air_date: String,
    //     #[serde(rename = "vote_average")]
    //     pub rating: f32,
    //     pub credits: Credits,
    //     pub genres: Vec<Genre>,
    //     pub poster_path: String,
    //     pub overview: String,
    // }

    #[derive(Deserialize, Debug)]
    pub struct Genre {
        pub name: String,
    }

    #[derive(Deserialize, Debug)]
    pub struct Credits {
        pub cast: Vec<Cast>,
        pub crew: Vec<Crew>,
    }

    #[derive(Deserialize, Debug)]
    pub struct Cast {
        pub name: String,
    }

    #[derive(Deserialize, Debug)]
    pub struct Crew {
        pub name: String,
        pub job: String,
    }
}
