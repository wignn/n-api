use axum::{
    extract::{Path, State},
    Extension, Json,
};
use axum::http::StatusCode;
use crate::{
    require_role, AppState, errors::AppError,
    middleware::auth::AuthUser,
    models::genre_model::{CreateGenreDto, GenreDto, UpdateGenreDto},
    models::response_model::ApiResponse,
    models::user_model::Role,
    services::genre_service::GenreService,
};
use tracing::{info, error, instrument};

pub struct GenreHandler;

impl GenreHandler {
    fn create_service(state: &AppState) -> GenreService {
        GenreService::new(state.db.clone())
    }


    #[instrument(skip(state, request), fields(
    user_id = %auth_user.id,
    user_role = ?auth_user.role,
    genre_name = ?request.title
    ))]
    pub async fn create_genre(
        State(state): State<AppState>,
        Extension(auth_user): Extension<AuthUser>,
        Json(request): Json<CreateGenreDto>,
    ) -> Result<(StatusCode, Json<ApiResponse<GenreDto>>), AppError> {
        info!("Attempting to create genre");
        require_role!(auth_user, Role::Admin);

        let service = Self::create_service(&state);

        match service.create_genre(request).await {
            Ok(genre) => {
                info!(
                    genre_id = %genre.id,
                    genre_title = %genre.title,
                    "Genre created successfully"
                );
                Ok((
                    StatusCode::CREATED,
                    Json(ApiResponse::with_message("Genre created successfully", genre)),
                ))
            }
            Err(e) => {
                error!(error = ?e, "Failed to create genre");
                Err(e)
            }
        }
    }

    pub async fn update_genre(
        State(state): State<AppState>,
        Extension(auth_user): Extension<AuthUser>,
        Path(id): Path<String>,
        Json(request): Json<UpdateGenreDto>,
    ) -> Result<(StatusCode, Json<ApiResponse<GenreDto>>), AppError> {
        info!("Attempting to update genre");
        require_role!(auth_user, Role::Admin);

        let service = Self::create_service(&state);

        match service.update_genre(id, request).await {
            Ok(genre) => {
                info!(
                    genre_id = %genre.id,
                    genre_title = %genre.title,
                    "Genre updated successfully"
                );
                Ok((
                    StatusCode::OK,
                    Json(ApiResponse::with_message("Genre updated successfully", genre)),
                ))
            }
            Err(e) => {
                error!(error = ?e, "Failed to update genre");
                Err(e)
            }
        }
    }


    #[instrument(skip(state), fields(
        user_id = %auth_user.id,
        user_role = ?auth_user.role,
        genre_id = %id
    ))]
    pub async fn delete_genre(
        State(state): State<AppState>,
        Extension(auth_user): Extension<AuthUser>,
        Path(id): Path<String>,
    ) -> Result<(StatusCode, Json<ApiResponse<()>>), AppError> {
        require_role!(auth_user, Role::Admin);

        let service = Self::create_service(&state);

        match service.delete_genre(id).await {
            Ok(_) => {
                info!("genre deleted successfully");
                Ok((
                    StatusCode::NO_CONTENT,
                    Json(ApiResponse::with_message("Genre deleted successfully", ())),
                ))
            }
            Err(e) => {
                error!(error = ?e, "Failed to delete genre");
                Err(e)
            }
        }
    }
    #[instrument(skip(state), fields(genre_id= %id))]
    pub async fn get_genre(
        State(state): State<AppState>,
        Path(id): Path<String>,
    ) -> Result<(StatusCode, Json<ApiResponse<GenreDto>>), AppError> {
        info!("Fetching single genre");
        let service = Self::create_service(&state);
        let genre = service.get_genre(id).await?;
        info!(genre_title = %genre.title, "Genre fetched successfully");

        Ok((StatusCode::OK, Json(ApiResponse::success(genre))))
    }

    #[tracing::instrument(
        name = "get_genres",
        skip(state),
    )]
    pub async fn get_genres(
        State(state): State<AppState>,
    ) -> Result<(StatusCode, Json<ApiResponse<Vec<GenreDto>>>), AppError> {
        info!("Fetching genres");
        let service = Self::create_service(&state);
        let genres = service.get_genres().await?;
        info!(count = genres.len(), "Genres fetched successfully");
        Ok((StatusCode::OK, Json(ApiResponse::success(genres))))
    }

}
