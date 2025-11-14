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
use tracing::{info, error, instrument};

pub struct BookHandler;

impl BookHandler {
    fn create_service(state: &AppState) -> BookService {
        BookService::new(state.db.clone())
    }

    #[instrument(skip(state), fields(
        page = %params.page,
        page_size = %params.page_size
    ))]
    pub async fn get_books(
        State(state): State<AppState>,
        Query(params): Query<PaginationParams>,
    ) -> Result<Json<PaginatedResponse<BookDto>>, AppError> {
        info!("Fetching books with pagination");

        let service = Self::create_service(&state);
        let paginated = service.get_books(params).await?;

        info!(
            total_items = paginated.total_items,
            total_pages = paginated.total_pages,
            "Books fetched successfully"
        );

        Ok(Json(paginated))
    }

    #[instrument(skip(state), fields(book_id = %id))]
    pub async fn get_book(
        State(state): State<AppState>,
        Path(id): Path<String>,
    ) -> Result<(StatusCode, Json<ApiResponse<BookDto>>), AppError> {
        info!("Fetching single book");

        let service = Self::create_service(&state);
        let book = service.get_book(id).await?;

        info!(book_title = %book.title, "Book fetched successfully");

        Ok((StatusCode::OK, Json(ApiResponse::success(book))))
    }

    #[instrument(skip(state, request), fields(
        user_id = %auth_user.id,
        user_role = ?auth_user.role,
        book_title = %request.title
    ))]
    pub async fn create_book(
        State(state): State<AppState>,
        Extension(auth_user): Extension<AuthUser>,
        Json(request): Json<CreateBookDto>,
    ) -> Result<(StatusCode, Json<ApiResponse<BookDto>>), AppError> {
        info!("Attempting to create book");

        require_role!(auth_user, Role::Admin);

        let service = Self::create_service(&state);

        match service.create_book(request).await {
            Ok(book) => {
                info!(
                    book_id = %book.id,
                    book_title = %book.title,
                    "Book created successfully"
                );
                Ok((
                    StatusCode::CREATED,
                    Json(ApiResponse::with_message("Book created successfully", book)),
                ))
            }
            Err(e) => {
                error!(error = ?e, "Failed to create book");
                Err(e)
            }
        }
    }

    #[instrument(skip(state, request), fields(
        user_id = %auth_user.id,
        user_role = ?auth_user.role,
        book_id = %id
    ))]
    pub async fn update_book(
        State(state): State<AppState>,
        Extension(auth_user): Extension<AuthUser>,
        Path(id): Path<String>,
        Json(request): Json<UpdateBookDto>,
    ) -> Result<(StatusCode, Json<ApiResponse<BookDto>>), AppError> {
        info!("Attempting to update book");

        require_role!(auth_user, Role::Admin);

        let service = Self::create_service(&state);

        match service.update_book(id, request).await {
            Ok(book) => {
                info!(
                    book_id = %book.id,
                    book_title = %book.title,
                    "Book updated successfully"
                );
                Ok((
                    StatusCode::OK,
                    Json(ApiResponse::with_message("Book updated successfully", book)),
                ))
            }
            Err(e) => {
                error!(error = ?e, "Failed to update book");
                Err(e)
            }
        }
    }

    #[instrument(skip(state), fields(
        user_id = %auth_user.id,
        user_role = ?auth_user.role,
        book_id = %id
    ))]
    pub async fn delete_book(
        State(state): State<AppState>,
        Extension(auth_user): Extension<AuthUser>,
        Path(id): Path<String>,
    ) -> Result<(StatusCode, Json<ApiResponse<()>>), AppError> {
        info!("Attempting to delete book");

        require_role!(auth_user, Role::Admin);

        let service = Self::create_service(&state);

        match service.delete_book(id).await {
            Ok(_) => {
                info!("Book deleted successfully");
                Ok((
                    StatusCode::NO_CONTENT,
                    Json(ApiResponse::with_message("Book deleted successfully", ())),
                ))
            }
            Err(e) => {
                error!(error = ?e, "Failed to delete book");
                Err(e)
            }
        }
    }
}