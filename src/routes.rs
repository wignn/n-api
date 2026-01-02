use crate::{
    handlers::{
        auth_handler::AuthHandler,
        book_handler::BookHandler,
        chapter_handler::ChapterHandler,
        genre_handler::GenreHandler,
        health_handler::{db_health_check, health_checker_handler},
        upload_handler::UploadHandler,
    },
    middleware::{api_key::api_key_middleware, auth::auth_middleware},
    AppState,
};
use axum::{
    middleware as axum_middleware,
    routing::{delete, get, post, put},
    Router,
};
use tower_cookies::CookieManagerLayer;
use tower_http::cors::CorsLayer;

pub fn create_routes(app_state: AppState, cors: CorsLayer) -> Router {
    Router::new()
        .nest("/api", api_routes(app_state.clone()))
        .route("/healthy", get(health_checker_handler))
        .route("/db-health", get(db_health_check))
        .with_state(app_state)
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(CookieManagerLayer::new())
        .layer(cors)
}

fn api_routes(app_state: AppState) -> Router<AppState> {
    Router::new()
        .nest("/auth", auth_routes(app_state.clone()))
        .merge(genre_routes(app_state.clone()))
        .merge(book_routes(app_state.clone()))
        .merge(chapter_routes(app_state.clone()))
        .merge(upload_routes(app_state.clone()))
}

fn auth_routes(app_state: AppState) -> Router<AppState> {
    let public = Router::new()
        .route("/register", post(AuthHandler::register))
        .route("/login", post(AuthHandler::login));

    let protected = Router::new()
        .route("/me", get(AuthHandler::me))
        .route("/refresh", post(AuthHandler::refresh_token))
        .route("/logout", post(AuthHandler::logout))
        .route_layer(axum_middleware::from_fn_with_state(
            app_state,
            auth_middleware,
        ));

    public.merge(protected)
}

fn genre_routes(app_state: AppState) -> Router<AppState> {
    let public = Router::new()
        .route("/genres", get(GenreHandler::get_genres))
        .route("/genre/{id}", get(GenreHandler::get_genre))
        .route_layer(axum_middleware::from_fn_with_state(
            app_state.clone(),
            api_key_middleware,
        ));

    let protected = Router::new()
        .route("/genre", post(GenreHandler::create_genre))
        .route(
            "/genre/{id}",
            put(GenreHandler::update_genre).delete(GenreHandler::delete_genre),
        )
        .route_layer(axum_middleware::from_fn_with_state(
            app_state,
            auth_middleware,
        ));

    public.merge(protected)
}

fn book_routes(app_state: AppState) -> Router<AppState> {
    let public = Router::new()
        .route("/books", get(BookHandler::get_books))
        .route("/book/{id}", get(BookHandler::get_book))
        .route_layer(axum_middleware::from_fn_with_state(
            app_state.clone(),
            api_key_middleware,
        ));

    let protected = Router::new()
        .route("/book", post(BookHandler::create_book))
        .route(
            "/book/{id}",
            put(BookHandler::update_book).delete(BookHandler::delete_book),
        )
        .route_layer(axum_middleware::from_fn_with_state(
            app_state,
            auth_middleware,
        ));

    public.merge(protected)
}

fn chapter_routes(app_state: AppState) -> Router<AppState> {
    let public = Router::new()
        .route("/chapters", get(ChapterHandler::get_chapters))
        .route(
            "/chapters/book/{book_id}",
            get(ChapterHandler::get_chapters_by_book),
        )
        .route("/chapter/{id}", get(ChapterHandler::get_chapter))
        .route_layer(axum_middleware::from_fn_with_state(
            app_state.clone(),
            api_key_middleware,
        ));

    let protected = Router::new()
        .route("/chapter", post(ChapterHandler::create_chapter))
        .route(
            "/chapter/{id}",
            put(ChapterHandler::update_chapter).delete(ChapterHandler::delete_chapter),
        )
        .route_layer(axum_middleware::from_fn_with_state(
            app_state,
            auth_middleware,
        ));

    public.merge(protected)
}

fn upload_routes(app_state: AppState) -> Router<AppState> {
    Router::new()
        .route("/upload/content", post(UploadHandler::upload_content))
        .route("/upload/{id}", get(UploadHandler::get_upload))
        .route("/upload/{id}", delete(UploadHandler::delete_upload))
        .route_layer(axum_middleware::from_fn_with_state(
            app_state,
            auth_middleware,
        ))
}
