use std::sync::Arc;
use libvips::VipsApp;
use tokio::net::TcpListener;
use tracing::{info, warn};

// App modules
mod config;
mod domain;
mod error;
mod web;

use crate::config::Config;


// Replace memory allocator to Jemalloc.
// To avoid fragmentation of memory
#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        // .json() 
        .init();

    info!("Initializing Image Converter Service...");
    // Load Config
    // If something is missing it should panic!
    let config = Arc::new(Config::from_env());
    info!("✅ Loaded Configuration: Host={}, Puerto={}, Threads={}", 
        config.host, config.port, config.concurrency_limit);
    
    // Initialize LIBVIPS
    // VipsApp must live during the whole app life.
    // concurrency=false: Tokio manages it with spawn_blocking
    // to avoid Vips saturating the runtime.
    let _vips_app = VipsApp::new("ImageConverter", false)
        .expect("CRITICAL ERROR: Couldn't start libvips");

    // Configure libvips cache
    domain::init_vips(&config);
    // Build the web app (web module)
    let app = web::app(config.clone());

    // Initialize TCP server
    let addr = config.socket_addr();
    let listener = TcpListener::bind(addr).await.expect("Could not bind the socket address");
    
    info!("Server listening on http://{}", addr);

    // Execute with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("Error executing the server");
}

/// Listens signals of the OS (CTRL+C, SIGTERM) to stop.
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Fail to install CTRL+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Fallo al instalar handler de SIGTERM")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    warn!("Shutdown signal received. Closing connections...");
}