use chrono::Utc;
use sqlx::QueryBuilder;
use crate::database::Database;
use crate::errors::AppResult;
use crate::models::genre_model::{CreateGenreDto, Genre, GenreDto, UpdateGenreDto};

pub struct GenreService {
    db: Database,
}


impl GenreService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }


    pub async fn get_genres(&self) -> AppResult<Vec<GenreDto>> {
        let redis = &self.db.redis;

        if let Ok(Some(cached_genre)) = redis.get_json::<Vec<GenreDto>>("genre:list").await {
            return Ok(cached_genre);
        }
        let genre = sqlx::query_as::<_, Genre>(
            r#"
        SELECT id, title, description, created_at, updated_at
        FROM "Genre"
        "#
        )
            .fetch_all(&self.db.pool)
            .await?;

        let genres: Vec<GenreDto> = genre.into_iter().map(Into::into).collect();

        let _ = redis.set_json(&"genre:list".to_string(), &genres, 600).await;

        Ok(genres)
    }
    pub async fn get_genre(&self, id: String) -> AppResult<GenreDto> {
        let redis = &self.db.redis;
        let cache_key = format!("genre:{id}");

        if let Ok(Some(cached_genre)) = redis.get_json::<GenreDto>(&cache_key).await {
            return Ok(cached_genre);
        }

        let genre = sqlx::query_as::<_, Genre>(
            r#"
    SELECT id, title, description, created_at, updated_at
    FROM "Genre" WHERE id = $1
    "#
        )
            .bind(&id)
            .fetch_one(&self.db.pool)
            .await?;


        let data: GenreDto = genre.into();
        let _ = redis.set_json(&cache_key, &data, 600).await;

        Ok(data)
    }
    pub async fn create_genre(&self, request: CreateGenreDto) -> AppResult<GenreDto> {
        let genre = sqlx::query_as::<_, Genre>(
            r#"
                    INSERT INTO "Genre" (id, title, description, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5)
                    RETURNING id, title, description, created_at, updated_at
                "#
        )
            .bind(cuid2::create_id())
            .bind(&request.title)
            .bind(&request.description)
            .bind(Utc::now())
            .bind(Utc::now())
            .fetch_one(&self.db.pool)
            .await?;

        Ok(genre.into())
    }



    pub async fn update_genre(&self, id: String, request: UpdateGenreDto) -> AppResult<GenreDto> {
        let redis = &self.db.redis;
        let cache_key = format!("genre:{id}");

        let mut builder = QueryBuilder::new(r#"UPDATE "Genre" SET "#);
        let mut separated = builder.separated(", ");
        let mut has_updates = false;

        if let Some(ref title) = request.title {
            separated.push("title").push_bind(title);
            has_updates = true;
        }

        if let Some(ref description) = request.description {
            separated.push("description").push_bind(description);
            has_updates = true;
        }

        if !has_updates {
            return self.get_genre(id).await;
        }

        separated.push("updated_at").push_bind(Utc::now());

        builder.push(" WHERE id = ").push_bind(id.clone());
        builder.push(" RETURNING *");

        let updated_genre = builder
            .build_query_as::<Genre>()
            .fetch_one(&self.db.pool)
            .await?;

        let _ = redis.del(&cache_key).await;
        let _ = redis.del_prefix("genre:list").await;

        Ok(updated_genre.into())
    }

    pub async fn delete_genre(&self, id: String) -> AppResult<GenreDto> {
        let redis = &self.db.redis;
        let cache_key = format!("genre:{id}");

        let deleted_genre = sqlx::query_as::<_, Genre>(
            r#"
            DELETE FROM "Genre"
            WHERE id = $1
            RETURNING id, title, description, created_at, updated_at
            "#
        )
            .bind(&id)
            .fetch_one(&self.db.pool)
            .await?;

        let _ = redis.del(&cache_key).await;
        let _ = redis.del_prefix("genre:list").await;

        Ok(deleted_genre.into())
    }
}