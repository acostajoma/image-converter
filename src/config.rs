use std::{
    env,
    net::SocketAddr
};

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub images_directory: String,
    pub concurrency_limit: usize,
    pub vips_cache_mb: u64,
}

impl Config {
    /**
     * Load configuration from environment variables 
     * If there are not, use default values
    */
    pub fn from_env() -> Self {
        let host = env::var("HOST").unwrap_or( "0.0.0.0".to_string());
        let port = env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok()) // Try to convert to u16
            .unwrap_or(3000); 
        let images_directory = env::var("IMAGES_DIR").unwrap_or("images".to_string());
        // Critical for performance
        // By default we se the number of CPUs available
        // This prevents overloading the cpu with more threads that it can handle
        let concurrency_limit = env::var("MAX_CONCURRENCY")
            .ok()
            .and_then(|c| c.parse().ok())
            .unwrap_or_else(|| {
                std::thread::available_parallelism()
                    .map(|n| n.get())
                    .unwrap_or(4) // Safe Fallback if we can't detect  the cpu cores 
            });

        let vips_cache_mb = env::var("VIPS_CACHE_MB")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(128);

        Self {
            host,
            port,
            images_directory,
            concurrency_limit,
            vips_cache_mb
        } 
    }

    /// Helper to generate the socket address compatible with axum.
    pub fn socket_addr(&self) -> SocketAddr {
        format!("{}:{}", self.host, self.port)
            .parse()
            .unwrap()
    }
    
}

