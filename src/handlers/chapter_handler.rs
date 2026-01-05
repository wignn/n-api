use crate::middleware::auth::AuthUser;
use crate::models::chapter_model::{ChapterDto, CreateChapterDto, UpdateChapterDto};
use crate::models::paging_model::{PaginatedResponse, PaginationParams};
use crate::models::response_model::ApiResponse;
use crate::models::user_model::Role;
use crate::require_role;
use crate::services::chapter_service::ChapterService;
use crate::{errors::AppError, AppState};
use axum::Extension;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use tracing::{error, info, instrument};

pub struct ChapterHandler;

impl ChapterHandler {
    fn create_service(state: &AppState) -> ChapterService {
        ChapterService::new(state.db.clone(), &state.notification)
    }

    #[instrument(skip(state), fields(
        page = %params.page,
        page_size = %params.page_size
    ))]
    pub async fn get_chapters(
        State(state): State<AppState>,
        Query(params): Query<PaginationParams>,
    ) -> Result<Json<PaginatedResponse<ChapterDto>>, AppError> {
        info!("Fetching chapters with pagination");
        let service = Self::create_service(&state);
        let paginated = service.get_chapters(params).await?;
        info!(
            total_items = paginated.total_items,
            total_pages = paginated.total_pages,
            "Chapters fetched successfully"
        );

        Ok(Json(paginated))
    }

    #[tracing::instrument(
        name = "get_chapters_by_book",
        skip(state),
        fields(
        book_id = %book_id,
        page = params.page,
        limit = params.page_size
        )
    )]
    pub async fn get_chapters_by_book(
        State(state): State<AppState>,
        Path(book_id): Path<String>,
        Query(params): Query<PaginationParams>,
    ) -> Result<Json<PaginatedResponse<ChapterDto>>, AppError> {
        info!("Fetching chapters for book");

        let service = Self::create_service(&state);
        let paginated = service
            .get_chapters_by_book(book_id.clone(), params)
            .await?;
        info!(
            total_chapters = paginated.total_items,
            "Chapters fetched successfully"
        );

        Ok(Json(paginated))
    }

    #[instrument(skip(state), fields(chapter_id = %id))]
    pub async fn get_chapter(
        State(state): State<AppState>,
        Path(id): Path<String>,
    ) -> Result<(StatusCode, Json<ApiResponse<ChapterDto>>), AppError> {
        info!("Fetching single chapter");
        let service = Self::create_service(&state);
        let chapter = service.get_chapter(id).await?;
        info!(chapter_title = %chapter.title, "chapter fetched successfully");
        Ok((StatusCode::OK, Json(ApiResponse::success(chapter))))
    }

    #[instrument(skip(state, request), fields(
        user_id = %auth_user.id,
        user_role = ?auth_user.role,
        chapter_title = %request.title
    ))]
    pub async fn create_chapter(
        State(state): State<AppState>,
        Extension(auth_user): Extension<AuthUser>,
        Json(request): Json<CreateChapterDto>,
    ) -> Result<(StatusCode, Json<ApiResponse<ChapterDto>>), AppError> {
        info!(user_role = ?auth_user.role, "Creating chapter by user");

        require_role!(auth_user, Role::Admin);
        let service = Self::create_service(&state);

        match service.create_chapter(request).await {
            Ok(chapter) => {
                info!(
                    chapter_id = %chapter.id,
                    chapter_title = %chapter.title,
                    "Chapter created successfully"
                );
                Ok((
                    StatusCode::CREATED,
                    Json(ApiResponse::with_message(
                        "Chapter created successfully",
                        chapter,
                    )),
                ))
            }
            Err(e) => {
                error!(error = ?e, "Failed to create chapter");
                Err(e)
            }
        }
    }

    #[instrument(skip(state, request), fields(
        chapter_id = %id,
        user_id = %auth_user.id,
        user_role = ?auth_user.role
    ))]
    pub async fn update_chapter(
        State(state): State<AppState>,
        Extension(auth_user): Extension<AuthUser>,
        Path(id): Path<String>,
        Json(request): Json<UpdateChapterDto>,
    ) -> Result<(StatusCode, Json<ApiResponse<ChapterDto>>), AppError> {
        info!(user_role = ?auth_user.role, "Updating chapter by user");
        require_role!(auth_user, Role::Admin);

        let service = Self::create_service(&state);

        match service.update_chapter(id, request).await {
            Ok(chapter) => {
                info!(
                    chapter_id = %chapter.id,
                    chapter_title = %chapter.title,
                    "Chapter updated successfully"
                );
                Ok((
                    StatusCode::OK,
                    Json(ApiResponse::with_message(
                        "Chapter updated successfully",
                        chapter,
                    )),
                ))
            }
            Err(e) => {
                error!(error = ?e, "Failed to update Chapter");
                Err(e)
            }
        }
    }

    #[instrument(skip(state), fields(
        chapter_id = %id,
        user_id = %auth_user.id,
        user_role = ?auth_user.role
    ))]
    pub async fn delete_chapter(
        State(state): State<AppState>,
        Extension(auth_user): Extension<AuthUser>,
        Path(id): Path<String>,
    ) -> Result<(StatusCode, Json<ApiResponse<()>>), AppError> {
        info!(user_role = ?auth_user.role, "Deleting chapter by user");
        require_role!(auth_user, Role::Admin);

        let service = Self::create_service(&state);

        match service.delete_chapter(id.clone()).await {
            Ok(_) => {
                info!(chapter_id = %id, "Chapter deleted successfully");
                Ok((
                    StatusCode::NO_CONTENT,
                    Json(ApiResponse::with_message(
                        "Chapter deleted successfully",
                        (),
                    )),
                ))
            }
            Err(e) => {
                error!(error = ?e, "Failed to delete Chapter");
                Err(e)
            }
        }
    }
}
