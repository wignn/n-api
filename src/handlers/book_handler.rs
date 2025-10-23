use crate::models::book_model::{CreateBookDto, UpdateBookDto};
use crate::models::paging::{PaginatedResponse, PaginationParams};
use crate::services::book_service::BookService;
use crate::utils::jwt::JwtService;
use crate::{models, AppState};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

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

    pub async fn create_book(
        State(state): State<AppState>,
        Json(request): Json<CreateBookDto>,
    ) -> Result<Json<models::book_model::Book>, StatusCode> {
        let jwt_service = JwtService::new(
            &state.config.jwt_secret_key,
            state.config.jwt_expire_in,
            state.config.jwt_refresh_expire_in,
        );

        let book_service = BookService::new(state.db.clone(), jwt_service);

        match book_service.create_book(request).await {
            Ok(book) => Ok(Json(book)),
            Err(e) => {
                eprintln!("Error creating book: {:?}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    pub async fn update_book(
        State(state): State<AppState>,
        Path(id): Path<Uuid>,
        Json(request): Json<UpdateBookDto>,
    ) -> Result<Json<models::book_model::Book>, StatusCode> {
        let jwt_service = JwtService::new(
            &state.config.jwt_secret_key,
            state.config.jwt_expire_in,
            state.config.jwt_refresh_expire_in,
        );

        let book_service = BookService::new(state.db.clone(), jwt_service);

        match book_service.update_book(id, request).await {
            Ok(book) => Ok(Json(book)),
            Err(e) => {
                eprintln!("Error creating book: {:?}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    pub async fn get_book(
        State(state): State<AppState>,

        Path(id): Path<Uuid>,
    ) -> Result<Json<models::book_model::Book>, StatusCode> {
        let jwt_service = JwtService::new(
            &state.config.jwt_secret_key,
            state.config.jwt_expire_in,
            state.config.jwt_refresh_expire_in,
        );

        let book_service = BookService::new(state.db.clone(), jwt_service);


        match book_service.get_book(id).await {
            Ok(book) => Ok(Json(book)),
            Err(e) => {
                eprintln!("Error creating book: {:?}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
    pub async fn delete_book(
        State(state): State<AppState>,
        Path(id): Path<Uuid>,
    ) -> Result<Json<models::book_model::Book>, StatusCode> {
        let jwt_service = JwtService::new(
            &state.config.jwt_secret_key,
            state.config.jwt_expire_in,
            state.config.jwt_refresh_expire_in,
        );

        let book_service = BookService::new(state.db.clone(), jwt_service);


        match book_service.delete_book(id).await {
            Ok(book) => Ok(Json(book)),
            Err(e) => {
                eprintln!("Error creating book: {:?}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}
