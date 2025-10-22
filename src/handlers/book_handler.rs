use crate::AppState;
use crate::models::book_model::{BookDto, CreateBookDto, UpdateBookDto};
use crate::models::paging::{PaginationParams, PaginatedResponse};
use crate::services::book_service::BookService;
use axum::{
    Json,
    http::StatusCode,
    extract::{State, Path, Query},
};

pub struct BookHandler;

impl BookHandler {
    pub async fn get_books(
        State(state): State<AppState>,
        Query(params): Query<PaginationParams>,
    ) -> Result<Json<PaginatedResponse<BookDto>>, StatusCode> {
        let book_service = BookService::new(state.db.clone());

        match book_service.get_books(params).await {
            Ok(response) => Ok(Json(response)),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }

    pub async fn get_book(
        State(state): State<AppState>,
        Path(id): Path<String>,
    ) -> Result<Json<BookDto>, StatusCode> {
        let book_service = BookService::new(state.db.clone());

        match book_service.get_book_by_id(&id).await {
            Ok(book) => Ok(Json(book)),
            Err(_) => Err(StatusCode::NOT_FOUND),
        }
    }

    pub async fn create_book(
        State(state): State<AppState>,
        Json(payload): Json<CreateBookDto>,
    ) -> Result<Json<BookDto>, StatusCode> {
        let book_service = BookService::new(state.db.clone());

        match book_service.create_book(payload).await {
            Ok(book) => Ok(Json(book)),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }

    pub async fn update_book(
        State(state): State<AppState>,
        Path(id): Path<String>,
        Json(payload): Json<UpdateBookDto>,
    ) -> Result<Json<BookDto>, StatusCode> {
        let book_service = BookService::new(state.db.clone());

        match book_service.update_book(&id, payload).await {
            Ok(book) => Ok(Json(book)),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }

    pub async fn delete_book(
        State(state): State<AppState>,
        Path(id): Path<String>,
    ) -> Result<StatusCode, StatusCode> {
        let book_service = BookService::new(state.db.clone());

        match book_service.delete_book(&id).await {
            Ok(_) => Ok(StatusCode::NO_CONTENT),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}