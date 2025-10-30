use crate::database::Database;
use crate::errors::AppResult;
use crate::models::book_model::{Book, BookDto, CreateBookDto, UpdateBookDto};
use crate::models::paging_model::{PaginatedResponse, PaginationParams};
use chrono::Utc;
use cuid2;
use sqlx::QueryBuilder;

pub struct BookService {
    db: Database,
}

impl BookService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub async fn create_book(&self, request: CreateBookDto) -> AppResult<BookDto> {
        let id = cuid2::create_id();

        let book = sqlx::query_as::<_, Book>(
            r#"
            INSERT INTO "Book" (
                id, title, author, cover, description, asset,
                status, language, release_date, popular,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING id, title, author, cover, description, asset,
                      status, language, release_date, popular,
                      created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(&request.title)
        .bind(&request.author)
        .bind(&request.cover)
        .bind(&request.description)
        .bind(&request.asset)
        .bind(request.status)
        .bind(&request.language)
        .bind(&request.release_date)
        .bind(request.popular)
        .bind(Utc::now())
        .bind(Utc::now())
        .fetch_one(&self.db.pool)
        .await?;

        Ok(book.into())
    }


    pub async fn get_books(&self, params: PaginationParams) -> AppResult<PaginatedResponse<BookDto>> {
        let offset = (params.page - 1) * params.page_size;
        let redis = &self.db.redis;

        let cache_key = format!(
            "books:list:page:{}:size:{}:search:{}:genres:{}",
            params.page,
            params.page_size,
            params.search.as_deref().unwrap_or(""),
            params.genres.as_deref().unwrap_or("")
        );

        if let Ok(Some(cached_response)) = redis.get_json::<PaginatedResponse<BookDto>>(&cache_key).await {
            return Ok(cached_response);
        }

        let mut where_conditions = Vec::new();
        let mut bind_index = 1;

        let search_pattern = params.search.as_ref().map(|s| format!("%{}%", s));

        if search_pattern.is_some() {
            where_conditions.push(format!("title ILIKE ${}", bind_index));
            bind_index += 1;
        }

        let genre_list: Option<Vec<String>> = params.genres.as_ref().map(|g| {
            g.split(',')
                .map(|s| s.trim().to_lowercase())
                .filter(|s| !s.is_empty())
                .collect()
        });

        let genre_count = genre_list.as_ref().map(|g| g.len()).unwrap_or(0);

        if genre_count > 0 {
            tracing::info!("Filtering by genres: {:?}", genre_list);

            let genre_placeholders: Vec<String> = (0..genre_count)
                .map(|i| format!("${}", bind_index + i))
                .collect();

            where_conditions.push(format!(
                r#"EXISTS (
                SELECT 1 FROM "BookGenre" bg
                INNER JOIN "Genre" g ON bg.genre_id = g.id
                WHERE bg.book_id = "Book".id
                AND LOWER(g.title) IN ({})
            )"#,
                genre_placeholders.join(", ")
            ));

            bind_index += genre_count;
        }

        let where_clause = if where_conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_conditions.join(" AND "))
        };

        tracing::info!("Generated WHERE clause: {}", where_clause);

        let count_query = format!(r#"SELECT COUNT(*) FROM "Book" {}"#, where_clause);
        let mut count_query_builder = sqlx::query_scalar::<_, i64>(&count_query);

        if let Some(ref pattern) = search_pattern {
            count_query_builder = count_query_builder.bind(pattern);
        }

        if let Some(ref genres) = genre_list {
            for genre in genres {
                count_query_builder = count_query_builder.bind(genre);
            }
        }

        let total_items = count_query_builder.fetch_one(&self.db.pool).await?;

        let fetch_query = format!(
            r#"
        SELECT id, title, author, cover, description, asset,
               status, language, release_date, popular,
               created_at, updated_at
        FROM "Book"
        {}
        ORDER BY created_at DESC
        LIMIT ${} OFFSET ${}
        "#,
            where_clause,
            bind_index,
            bind_index + 1
        );

        let mut fetch_query_builder = sqlx::query_as::<_, Book>(&fetch_query);

        if let Some(ref pattern) = search_pattern {
            fetch_query_builder = fetch_query_builder.bind(pattern);
        }

        if let Some(ref genres) = genre_list {
            for genre in genres {
                fetch_query_builder = fetch_query_builder.bind(genre);
            }
        }

        fetch_query_builder = fetch_query_builder
            .bind(params.page_size)
            .bind(offset);

        let books = fetch_query_builder.fetch_all(&self.db.pool).await?;

        let data: Vec<BookDto> = books.into_iter().map(BookDto::from).collect();
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

    pub async fn get_book(&self, id: String) -> AppResult<BookDto> {
        let redis = &self.db.redis;
        let cache_key = format!("book:{id}");

        if let Ok(Some(cached_book)) = redis.get_json::<BookDto>(&cache_key).await {
            return Ok(cached_book);
        }

        let book = sqlx::query_as::<_, Book>(
            r#"
            SELECT id, title, author, cover, description, asset, status, language, release_date, popular,
                   created_at, updated_at
            FROM "Book" WHERE id = $1
            "#,
        )
        .bind(&id)
        .fetch_one(&self.db.pool)
        .await?;

        let data: BookDto = book.into();
        let _ = redis.set_json(&cache_key, &data, 600).await;
 
        Ok(data)
    }

    pub async fn update_book(&self, id: String, request: UpdateBookDto) -> AppResult<BookDto> {
        let redis = &self.db.redis;
        let cache_key = format!("book:{id}");

        let mut builder = QueryBuilder::new(r#"UPDATE "Book" SET "#);
        let mut separated = builder.separated(", ");
        let mut has_updates = false;

        if let Some(ref title) = request.title {
            separated.push("title = ").push_bind_unseparated(title);
            has_updates = true;
        }
        if let Some(ref author) = request.author {
            separated.push("author = ").push_bind_unseparated(author);
            has_updates = true;
        }
        if let Some(ref cover) = request.cover {
            separated.push("cover = ").push_bind_unseparated(cover);
            has_updates = true;
        }
        if let Some(ref description) = request.description {
            separated
                .push("description = ")
                .push_bind_unseparated(description);
            has_updates = true;
        }
        if let Some(ref asset) = request.asset {
            separated.push("asset = ").push_bind_unseparated(asset);
            has_updates = true;
        }
        if let Some(ref status) = request.status {
            separated.push("status = ").push_bind_unseparated(status);
            has_updates = true;
        }
        if let Some(ref language) = request.language {
            separated
                .push("language = ")
                .push_bind_unseparated(language);
            has_updates = true;
        }
        if let Some(ref release_date) = request.release_date {
            separated
                .push("release_date = ")
                .push_bind_unseparated(release_date);
            has_updates = true;
        }
        if let Some(ref popular) = request.popular {
            separated.push("popular = ").push_bind_unseparated(popular);
            has_updates = true;
        }

        if !has_updates {
            return self.get_book(id).await;
        }

        separated
            .push("updated_at = ")
            .push_bind_unseparated(Utc::now());
        builder.push(" WHERE id = ").push_bind(id);
        builder.push(" RETURNING *");

        let updated_book = builder
            .build_query_as::<Book>()
            .fetch_one(&self.db.pool)
            .await?;

        redis.del(&cache_key).await.ok();
        let data:BookDto = updated_book.into();
        let _ = redis.del_prefix("books:list:").await;
        Ok(data)
    }

    pub async fn delete_book(&self, id: String) -> AppResult<BookDto> {
        let redis = &self.db.redis;
        let book = self.get_book(id.clone()).await?;

        sqlx::query(r#"DELETE FROM "Book" WHERE id = $1"#)
            .bind(&id)
            .execute(&self.db.pool)
            .await?;

        let redis_key = format!("book:{id}");
        if redis.exists(&redis_key).await.unwrap_or(false) {
            let _ = redis.del(&redis_key).await;
        }
        let data = book.into();
        Ok(data)
    }
}
