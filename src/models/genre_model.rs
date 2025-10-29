use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct Genre {
    pub id: String,
    pub title: String,
    pub description: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenreDto {
    pub id: String,
    pub title: String,
    pub description: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Deserialize, Clone)]
pub struct CreateGenreDto {
    pub title: Option<String>,
    pub description:Option<String>,
}


#[derive(Deserialize, Clone)]
pub struct UpdateGenreDto {
    pub title: Option<String>,
    pub description:Option<String>,
}

impl From<Genre> for GenreDto {
    fn from(genre: Genre) -> Self {
        Self {
            id: genre.id,
            title: genre.title,
            description: genre.description,
            created_at: genre.created_at,
            updated_at: genre.updated_at,
        }
    }
}
