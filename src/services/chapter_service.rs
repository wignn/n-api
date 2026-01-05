use crate::database::Database;
use crate::errors::AppResult;
use crate::models::chapter_model::{Chapter, ChapterDto, CreateChapterDto, UpdateChapterDto};
use crate::models::paging_model::{PaginatedResponse, PaginationParams};
use chrono::Utc;
use cuid2;
use sqlx::QueryBuilder;

pub struct ChapterService {
    db: Database,
}

impl ChapterService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub async fn create_chapter(&self, request: CreateChapterDto) -> AppResult<ChapterDto> {
        let chapter = sqlx::query_as::<_, Chapter>(
            r#"
            INSERT INTO "Chapter" (
                id, title, book_id, description, content, chapter_num,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, title, book_id, description, created_at, updated_at, content, chapter_num
            "#,
        )
        .bind(cuid2::create_id())
        .bind(&request.title)
        .bind(&request.book_id)
        .bind(&request.description)
        .bind(&request.content)
        .bind(request.chapter_num)
        .bind(Utc::now())
        .bind(Utc::now())
        .fetch_one(&self.db.pool)
        .await?;

        // Update the book's updated_at timestamp
        sqlx::query(r#"UPDATE "Book" SET updated_at = $1 WHERE id = $2"#)
            .bind(Utc::now())
            .bind(&request.book_id)
            .execute(&self.db.pool)
            .await?;

        let redis = &self.db.redis;
        let _ = redis
            .del_prefix(&format!("chapters:book:{}", request.book_id))
            .await;
        // Also invalidate book cache
        let _ = redis.del(&format!("book:{}", request.book_id)).await;
        let _ = redis.del_prefix("books:").await;

        Ok(chapter.into())
    }

    pub async fn get_chapters(
        &self,
        params: PaginationParams,
    ) -> AppResult<PaginatedResponse<ChapterDto>> {
        let offset = (params.page - 1) * params.page_size;
        let redis = &self.db.redis;

        let cache_key = format!(
            "chapters:list:page:{}:size:{}:search:{}",
            params.page,
            params.page_size,
            params.search.as_deref().unwrap_or("")
        );

        if let Ok(Some(cached_response)) = redis
            .get_json::<PaginatedResponse<ChapterDto>>(&cache_key)
            .await
        {
            return Ok(cached_response);
        }

        let mut where_conditions = Vec::new();
        let search_pattern = params.search.as_ref().map(|s| format!("%{}%", s));

        if search_pattern.is_some() {
            where_conditions.push("title ILIKE $1".to_string());
        }

        let where_clause = if where_conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_conditions.join(" AND "))
        };

        let count_query = format!(r#"SELECT COUNT(*) FROM "Chapter" {}"#, where_clause);
        let mut count_query_builder = sqlx::query_scalar::<_, i64>(&count_query);

        if let Some(ref pattern) = search_pattern {
            count_query_builder = count_query_builder.bind(pattern);
        }

        let total_items = count_query_builder.fetch_one(&self.db.pool).await?;

        let fetch_query = format!(
            r#"
            SELECT id, title, book_id, description, created_at, updated_at, content, chapter_num
            FROM "Chapter"
            {}
            ORDER BY chapter_num ASC
            LIMIT ${} OFFSET ${}
            "#,
            where_clause,
            if search_pattern.is_some() { 2 } else { 1 },
            if search_pattern.is_some() { 3 } else { 2 }
        );

        let mut fetch_query_builder = sqlx::query_as::<_, Chapter>(&fetch_query);

        if let Some(ref pattern) = search_pattern {
            fetch_query_builder = fetch_query_builder.bind(pattern);
        }

        fetch_query_builder = fetch_query_builder.bind(params.page_size).bind(offset);

        let chapters = fetch_query_builder.fetch_all(&self.db.pool).await?;

        let data: Vec<ChapterDto> = chapters.into_iter().map(ChapterDto::from).collect();
        let total_pages = (total_items as f64 / params.page_size as f64).ceil() as i64;

        let response = PaginatedResponse {
            data,
            page: params.page,
            page_size: params.page_size,
            total_items,
            total_pages,
        };

        let _ = redis.set_json(&cache_key, &response, 600).await;

        Ok(response)
    }

    pub async fn get_chapters_by_book(
        &self,
        book_id: String,
        params: PaginationParams,
    ) -> AppResult<PaginatedResponse<ChapterDto>> {
        let offset = (params.page - 1) * params.page_size;
        let redis = &self.db.redis;

        let cache_key = format!(
            "chapters:book:{}:page:{}:size:{}",
            book_id, params.page, params.page_size
        );

        if let Ok(Some(cached_response)) = redis
            .get_json::<PaginatedResponse<ChapterDto>>(&cache_key)
            .await
        {
            return Ok(cached_response);
        }

        let total_items =
            sqlx::query_scalar::<_, i64>(r#"SELECT COUNT(*) FROM "Chapter" WHERE book_id = $1"#)
                .bind(&book_id)
                .fetch_one(&self.db.pool)
                .await?;

        let chapters = sqlx::query_as::<_, Chapter>(
            r#"
            SELECT id, title, book_id, description, created_at, updated_at, content, chapter_num
            FROM "Chapter"
            WHERE book_id = $1
            ORDER BY chapter_num ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(&book_id)
        .bind(params.page_size)
        .bind(offset)
        .fetch_all(&self.db.pool)
        .await?;

        let data: Vec<ChapterDto> = chapters.into_iter().map(ChapterDto::from).collect();
        let total_pages = (total_items as f64 / params.page_size as f64).ceil() as i64;

        let response = PaginatedResponse {
            data,
            page: params.page,
            page_size: params.page_size,
            total_items,
            total_pages,
        };

        let _ = redis.set_json(&cache_key, &response, 600).await;

        Ok(response)
    }

    pub async fn get_chapter(&self, id: String) -> AppResult<ChapterDto> {
        let redis = &self.db.redis;
        let cache_key = format!("chapter:{}", id);

        if let Ok(Some(cached_chapter)) = redis.get_json::<ChapterDto>(&cache_key).await {
            return Ok(cached_chapter);
        }

        let chapter = sqlx::query_as::<_, Chapter>(
            r#"
            SELECT id, title, book_id, description, created_at, updated_at, content, chapter_num
            FROM "Chapter"
            WHERE id = $1
            "#,
        )
        .bind(&id)
        .fetch_one(&self.db.pool)
        .await?;

        let data: ChapterDto = chapter.into();
        let _ = redis.set_json(&cache_key, &data, 600).await;

        Ok(data)
    }

    pub async fn update_chapter(
        &self,
        id: String,
        request: UpdateChapterDto,
    ) -> AppResult<ChapterDto> {
        let redis = &self.db.redis;
        let cache_key = format!("chapter:{}", id);

        let mut builder = QueryBuilder::new(r#"UPDATE "Chapter" SET "#);
        let mut separated = builder.separated(", ");
        let mut has_updates = false;

        if let Some(ref title) = request.title {
            separated.push("title = ").push_bind_unseparated(title);
            has_updates = true;
        }
        if let Some(ref description) = request.description {
            separated
                .push("description = ")
                .push_bind_unseparated(description);
            has_updates = true;
        }
        if let Some(ref content) = request.content {
            separated.push("content = ").push_bind_unseparated(content);
            has_updates = true;
        }
        if let Some(ref chapter_num) = request.chapter_num {
            separated
                .push("chapter_num = ")
                .push_bind_unseparated(chapter_num);
            has_updates = true;
        }

        if !has_updates {
            return self.get_chapter(id).await;
        }

        separated
            .push("updated_at = ")
            .push_bind_unseparated(Utc::now());
        builder.push(" WHERE id = ").push_bind(&id);
        builder.push(" RETURNING *");

        let updated_chapter = builder
            .build_query_as::<Chapter>()
            .fetch_one(&self.db.pool)
            .await?;

        let book_id = updated_chapter.book_id.clone();
        redis.del(&cache_key).await.ok();
        let _ = redis.del_prefix("chapters:list:").await;
        let _ = redis
            .del_prefix(&format!("chapters:book:{}", book_id))
            .await;

        let data: ChapterDto = updated_chapter.into();
        Ok(data)
    }

    pub async fn delete_chapter(&self, id: String) -> AppResult<ChapterDto> {
        let redis = &self.db.redis;
        let chapter = self.get_chapter(id.clone()).await?;

        sqlx::query(r#"DELETE FROM "Chapter" WHERE id = $1"#)
            .bind(&id)
            .execute(&self.db.pool)
            .await?;

        let cache_key = format!("chapter:{}", id);
        if redis.exists(&cache_key).await.unwrap_or(false) {
            let _ = redis.del(&cache_key).await;
        }
        let _ = redis
            .del_prefix(&format!("chapters:book:{}", chapter.book_id))
            .await;

        Ok(chapter)
    }
}
