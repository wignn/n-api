use crate::middleware::auth::AuthUser;
use crate::models::user_model::Role;
use crate::models::book_model::{BookDto, CreateBookDto, UpdateBookDto};
use crate::models::paging_model::{PaginatedResponse, PaginationParams};
use crate::models::response_model::ApiResponse;
use crate::require_role;
use crate::services::book_service::BookService;
use crate::{errors::AppError, AppState};
use axum::Extension;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use tracing::info;

pub struct BookHandler;

impl BookHandler {
    fn create_service(state: &AppState) -> BookService {
        BookService::new(state.db.clone())
    }

    pub async fn get_books(
        State(state): State<AppState>,
        Query(params): Query<PaginationParams>,
    ) -> Result<Json<PaginatedResponse<BookDto>>, AppError> {
        let service = Self::create_service(&state);
        let paginated = service.get_books(params).await?;
        Ok(Json(paginated))
    }

    pub async fn get_book(
        State(state): State<AppState>,
        Path(id): Path<String>,
    ) -> Result<(StatusCode, Json<ApiResponse<BookDto>>), AppError> {
        let service = Self::create_service(&state);
        let book = service.get_book(id).await?;
        Ok((StatusCode::OK, Json(ApiResponse::success(book))))
    }

    pub async fn create_book(
        State(state): State<AppState>,
        Extension(auth_user): Extension<AuthUser>,
        Json(request): Json<CreateBookDto>,
    ) -> Result<(StatusCode, Json<ApiResponse<BookDto>>), AppError> {
        info!(user_role = ?auth_user.role, "Creating book by user");
        require_role!(auth_user, Role::Admin);

        let service = Self::create_service(&state);
        let book = service.create_book(request).await?;

        Ok((
            StatusCode::CREATED,
            Json(ApiResponse::with_message("Book created successfully", book)),
        ))
    }

    pub async fn update_book(
        State(state): State<AppState>,
        Extension(auth_user): Extension<AuthUser>,
        Path(id): Path<String>,
        Json(request): Json<UpdateBookDto>,
    ) -> Result<(StatusCode, Json<ApiResponse<BookDto>>), AppError> {
        info!(user_role = ?auth_user.role, "Updating book by user");
        require_role!(auth_user, Role::Admin);

        let service = Self::create_service(&state);
        let book = service.update_book(id, request).await?;

        Ok((
            StatusCode::OK,
            Json(ApiResponse::with_message("Book updated successfully", book)),
        ))
    }

    pub async fn delete_book(
        State(state): State<AppState>,
        Extension(auth_user): Extension<AuthUser>,
        Path(id): Path<String>,
    ) -> Result<(StatusCode, Json<ApiResponse<()>>), AppError> {
        info!(user_role = ?auth_user.role, "Deleting book by user");
        require_role!(auth_user, Role::Admin);

        let service = Self::create_service(&state);
        service.delete_book(id).await?;

        Ok((
            StatusCode::NO_CONTENT,
            Json(ApiResponse::with_message("Book deleted successfully", ())),
        ))
    }
}
