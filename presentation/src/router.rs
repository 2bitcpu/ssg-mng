use crate::{
    handler::{auth, content, public_handler},
    middleware::auth::auth_guard,
};
use application::UseCaseModule;
use axum::{
    Router,
    extract::DefaultBodyLimit,
    http::{HeaderValue, Method, header},
    middleware::from_fn_with_state,
    routing::{any, delete, get, get_service, post, put},
};
use config::CONFIG;
use std::sync::Arc;
use tower_http::{cors::CorsLayer, services::ServeDir};

#[allow(dead_code)]
pub fn create_router(usecases: Arc<dyn UseCaseModule>) -> Router {
    let content_router = Router::new()
        .route("/content", post(content::create))
        .route("/content/search", post(content::search))
        .route("/content/search", get(content::search_query))
        .route("/content/{id}", get(content::find))
        .route("/content/{id}", delete(content::remove))
        .route("/content", put(content::edit))
        .route("/content/tags/{limit}", get(content::tags))
        .route("/content/categories/{limit}", get(content::caregories))
        .layer(from_fn_with_state(usecases.clone(), auth_guard));

    let mut auth_router = Router::new()
        .route("/auth/signin", post(auth::signin))
        .route("/auth/signout", any(auth::signout));
    if CONFIG.security.allow_signup {
        auth_router = auth_router.route("/auth/signup", post(auth::signup));
    }

    let public_router = Router::new()
        .route("/content/{id}", get(public_handler::find))
        .route("/content/search", post(public_handler::search))
        .route("/content/search", get(public_handler::search_query))
        .route("/content/tags/{limit}", get(public_handler::tags))
        .route(
            "/content/caregories/{limit}",
            get(public_handler::caregories),
        );

    let manage_router = Router::new()
        .nest("/manage", content_router)
        .nest("/manage", auth_router);

    let mut app = Router::new()
        .nest("/service", manage_router)
        .nest("/service", public_router)
        .with_state(usecases);

    if !CONFIG.server.cors.is_empty() {
        let cors = CorsLayer::new()
            .allow_headers([
                header::CONTENT_TYPE,
                header::AUTHORIZATION,
                "DeviceId".parse().unwrap(),
            ])
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
            .allow_origin(
                config::CONFIG
                    .server
                    .cors
                    .iter()
                    .map(|s| s.parse::<HeaderValue>().unwrap())
                    .collect::<Vec<_>>(),
            );
        app = app.layer(cors);
    }
    app = app.layer(DefaultBodyLimit::max(1024 * 1024));

    if let Some(dir) = CONFIG.server.static_dir.as_ref() {
        app.fallback(get_service(ServeDir::new(dir)))
    } else {
        app
    }
}
