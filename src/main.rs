
mod schema;
use sliet_techfest_backend::routes::setup_routes;
use sliet_techfest_backend::state::SiteState;

use std::env;

use dotenvy::dotenv;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    let _ = dotenv();
    pretty_env_logger::init();
    tokio_rustls::rustls::crypto::aws_lc_rs::default_provider().install_default().unwrap();
    let frontend_url = env::var("FRONTEND_URL").unwrap();
    let state = &mut SiteState::init().await.unwrap();
    let routes = setup_routes()
        .with_state(state.clone())
        .layer(
            CorsLayer::permissive()
                .allow_headers([
                    http::header::COOKIE,
                    http::header::ACCESS_CONTROL_ALLOW_ORIGIN,
                    http::header::ORIGIN,
                    http::header::ALLOW,
                    http::header::CONTENT_TYPE,
                    http::header::ACCEPT_ENCODING,
                    http::header::CACHE_CONTROL,
                ])
                .allow_methods([
                    http::Method::GET,
                    http::Method::POST,
                    http::Method::PUT,
                    http::Method::PATCH,
                    http::Method::DELETE,
                ])
                .expose_headers([http::header::CONTENT_TYPE])
                .allow_credentials(true)
                .allow_origin([
                    frontend_url.parse().unwrap(),
                ]),
        )
        .layer(CompressionLayer::new().br(true).zstd(true).gzip(true));
    log::info!("Binding to port 3000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind to address");
    log::info!("Listening on all ip addresses");
    axum::serve(listener, routes)
        .await
        .expect("Failed while serving");
}
