use crate::database::Database;
use crate::errors::AppResult;
use crate::models::chapter_model::{ChapterDto, CreateChapterDto};

pub struct ChapterService {
    db: Database
}

impl ChapterService {
    pub fn new(db: Database) ->Self {Self{db}}

    // pub async fn create_chapter(&self, request: CreateChapterDto) -> AppResult<ChapterDto> {
    //     let redis = &self.db.redis;
    //
    //
    //
    // }
}