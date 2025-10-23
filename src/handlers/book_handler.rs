use crate::models::book_model::{Book, CreateBookDto, UpdateBookDto};
use crate::models::paging::{PaginatedResponse, PaginationParams};
use crate::models::response_model::ApiResponse;
use crate::services::book_service::BookService;
use crate::{errors::AppError, AppState};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

pub struct BookHandler;

impl BookHandler {
    fn create_service(state: &AppState) -> BookService {
        BookService::new(state.db.clone())
    }

    pub async fn get_books(
        State(state): State<AppState>,
        Query(params): Query<PaginationParams>,
    ) -> Result<Json<PaginatedResponse<Book>>, AppError> {
        let service = Self::create_service(&state);
        let paginated = service.get_books(params).await?;
        Ok(Json(paginated))
    }

    pub async fn create_book(
        State(state): State<AppState>,
        Json(request): Json<CreateBookDto>,
    ) -> Result<impl IntoResponse, AppError> {
        let service = Self::create_service(&state);
        let book = service.create_book(request).await?;
        
        Ok((
            StatusCode::CREATED,
            Json(ApiResponse::with_message("Book created successfully", book)),
        ))
    }

    pub async fn get_book(
        State(state): State<AppState>,
        Path(id): Path<String>,
    ) -> Result<Json<ApiResponse<Book>>, AppError> {
        let service = Self::create_service(&state);
        let book = service.get_book(id).await?;
        Ok(Json(ApiResponse::success(book)))
    }

    pub async fn update_book(
        State(state): State<AppState>,
        Path(id): Path<String>,
        Json(request): Json<UpdateBookDto>,
    ) -> Result<Json<ApiResponse<Book>>, AppError> {
        let service = Self::create_service(&state);
        let book = service.update_book(id, request).await?;
        Ok(Json(ApiResponse::with_message("Book updated successfully", book)))
    }

    pub async fn delete_book(
        State(state): State<AppState>,
        Path(id): Path<String>,
    ) -> Result<StatusCode, AppError> {
        let service = Self::create_service(&state);
        service.delete_book(id).await?;
        Ok(StatusCode::NO_CONTENT)
    }
}