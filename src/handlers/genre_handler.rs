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

pub struct GenreHandler;

impl GenreHandler {
    fn create_service(state: &AppState) -> GenreService {
        GenreService::new(state.db.clone())
    }

    pub async fn create_genre(
        State(state): State<AppState>,
        Extension(auth_user): Extension<AuthUser>,
        Json(request): Json<CreateGenreDto>,
    ) -> Result<(StatusCode, Json<ApiResponse<GenreDto>>), AppError> {
        require_role!(auth_user, Role::Admin);

        let service = Self::create_service(&state);
        let genre = service.create_genre(request).await?;

        Ok((
            StatusCode::CREATED,
            Json(ApiResponse::with_message("Genre created successfully", genre)),
        ))
    }

    pub async fn update_genre(
        State(state): State<AppState>,
        Extension(auth_user): Extension<AuthUser>,
        Path(id): Path<String>,
        Json(request): Json<UpdateGenreDto>,
    ) -> Result<(StatusCode, Json<ApiResponse<GenreDto>>), AppError> {
        require_role!(auth_user, Role::Admin);

        let service = Self::create_service(&state);
        let genre = service.update_genre(request, id).await?;

        Ok((
            StatusCode::OK,
            Json(ApiResponse::with_message("Genre updated successfully", genre)),
        ))
    }

    pub async fn delete_genre(
        State(state): State<AppState>,
        Extension(auth_user): Extension<AuthUser>,
        Path(id): Path<String>,
    ) -> Result<(StatusCode, Json<ApiResponse<()>>), AppError> {
        require_role!(auth_user, Role::Admin);

        let service = Self::create_service(&state);
        service.delete_genre(id).await?;

        Ok((
            StatusCode::NO_CONTENT,
            Json(ApiResponse::with_message("Genre deleted successfully", ())),
        ))
    }

    pub async fn get_genre(
        State(state): State<AppState>,
        Path(id): Path<String>,
    ) -> Result<(StatusCode, Json<ApiResponse<GenreDto>>), AppError> {
        let service = Self::create_service(&state);
        let genre = service.get_genre(id).await?;

        Ok((StatusCode::OK, Json(ApiResponse::success(genre))))
    }

    pub async fn get_genres(
        State(state): State<AppState>,
    ) ->Result<(StatusCode, Json<ApiResponse<Vec<GenreDto>>>), AppError> {
        let service = Self::create_service(&state);
        let genres = service.get_genres().await?;
        Ok((StatusCode::OK, Json(ApiResponse::success(genres))))
    }


}
