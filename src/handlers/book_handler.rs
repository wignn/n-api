use crate::models::book_model::{BookDto, CreateBookDto, UpdateBookDto};
use crate::models::paging::{PaginatedResponse, PaginationParams};
use crate::services::book_service::BookService;
use crate::utils::jwt::JwtService;
use crate::{models, AppState};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};

pub struct BookHandler;

impl BookHandler {
    pub async fn get_books(
        State(state): State<AppState>,
        Query(params): Query<PaginationParams>,
    ) -> Result<Json<PaginatedResponse<models::book_model::Book>>, StatusCode> {
        let jwt_service = JwtService::new(
            &state.config.jwt_secret_key,
            state.config.jwt_expire_in,
            state.config.jwt_refresh_expire_in,
        );

        let book_service = BookService::new(state.db.clone(), jwt_service);

        match book_service.get_books(params).await {
            Ok(paginated) => Ok(Json(paginated)),
            Err(e) => {
                eprintln!("Error getting books: {:?}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}
