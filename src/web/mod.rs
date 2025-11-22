use std::sync::Arc;
use std::time::Duration;

use axum::{routing::get, Router};
use tokio::sync::Semaphore;
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};

use crate::config::Config;
mod handlers;

/// Constructs the main Router of the App
pub fn app(config: Arc<Config>) -> Router {
    // Initialize Global Semaphore
    // Limit how many operations can VIPS run at the time
    let semaphore = Arc::new(Semaphore::new(config.concurrency_limit));

    let state = (config, semaphore);

    // CORS Middleware  (Adjust for PROD)
    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_origin(Any)
        .allow_headers(Any);

    Router::new()
        // Health Check (useful for Kubernetes/Load Balancers)
        .route("/health", get(|| async { "OK" }))
        
        // Main route for the images
        .route("/images/{id}", get(handlers::handle_image))
        
        // Inject the state on the handler
        .with_state(state)
        
        // Middlewares Stack  (Executed bottom up)
        .layer(
            ServiceBuilder::new()
                // Detailed logs for each request
                .layer(TraceLayer::new_for_http())
                // Hard Timeout of 5 seconds, cut the connection after it
                .layer(TimeoutLayer::new(Duration::from_secs(5)))
                // 3. Compress Gzip/Brotli (headers, JSONs, etc)
                // Note: Images are already compressed. This is for error and texts.
                .layer(CompressionLayer::new())
                // CORS
                .layer(cors)
        )
}