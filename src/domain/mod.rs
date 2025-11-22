use tracing::info;
use crate::config::Config;

pub mod image_processing;

/**
 * Configure the global params of LibVips
 * Must be called after initializing Libvips
 * And before processing any image
 */
pub fn init_vips(config: &Config) {
    let cache_bytes = config.vips_cache_mb * 1024 * 1024;

    /*
    * SAFETY: These are direct FFI calls to the underlying C library (libvips).
    * They are required because configuration functions are not exposed by the safe Rust wrapper.
    * This is safe because we are only passing primitive integers to set global limits, with no raw pointer manipulation or memory allocation involved.
    */ 
    unsafe {
        libvips::bindings::vips_cache_set_max_mem(cache_bytes);
        libvips::bindings::vips_cache_set_max_files(50);
        libvips::bindings::vips_cache_set_max(500);
    }

    info!("✅ Vips Cache configured to: {} MB", config.vips_cache_mb);
}