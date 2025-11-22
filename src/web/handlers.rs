use std::collections::hash_map::DefaultHasher;
use std::{
    fs,
    hash::{Hash, Hasher},
    sync::Arc,
};
use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse
};
use serde::Deserialize;
use tokio::sync::Semaphore;
use tracing::{info, instrument};
use crate::{
    config::Config,
    domain::image_processing::{self,ProcessOptions},
    error::AppError
};

// Ordered by priority
static  PRIORITIZED_FORMATS: &[(&str, &str)] = &[
    (".avif", "image/avif"),
    (".webp", "image/webp"),
    (".png",  "image/png"),
    (".jpg",  "image/jpeg"),
];

#[derive(Debug, Deserialize)]
pub struct ResizeParams {
    width: Option<i32>,
    height: Option<i32>,
}

/// handle_image is the main handler to obtain and convert images
// #[instrument] generate logs with function arguments
#[instrument(skip(headers, semaphore, config))]
pub async fn handle_image(
    Path(image_id): Path<String>,
    Query(params): Query<ResizeParams>,
    headers: HeaderMap,
    State((config, semaphore)): State<(Arc<Config>, Arc<Semaphore>)>,
    ) -> Result<impl IntoResponse, AppError> {
        let source_path = format!("{}/{}.png", config.images_directory, image_id);
        let metadata = fs::metadata(&source_path)
            .map_err(|_| AppError::NotFound)?;

        let mut chosen_format = ".jpg"; // Default Format
        let mut chosen_mime = "image/jpeg"; // Default Mime"

        if let Some(accept) = headers.get(header::ACCEPT)
            .and_then(|h| h.to_str().ok()) {
                for (suffix, mime) in PRIORITIZED_FORMATS {
                    if accept.contains(mime){
                        chosen_format = *suffix;
                        chosen_mime = *mime;
                        break;
                    }
                }
            }

        // E-Tag Generation
        let mut hasher = DefaultHasher::new();
        metadata.modified()?.hash(&mut hasher);
        metadata.len().hash(&mut hasher);
        chosen_format.hash(&mut hasher);
        params.width.hash(&mut hasher);
        params.height.hash(&mut hasher);
        "v1".hash(&mut hasher); // Version of the algorithm
        
        let etag_value = format!("\"{:x}\"", hasher.finish());

        // Check Browser Cache
        if let Some(if_none_match) = headers.get(header::IF_NONE_MATCH){
            if let Ok(val) = if_none_match.to_str(){
                if val.trim() == etag_value {
                    // Cache Hit: The client already has the image.
                    let mut cache_headers = HeaderMap::new();
                    if let Ok(etag_header_value) = HeaderValue::from_str(&etag_value) {
                        cache_headers.insert(header::ETAG, etag_header_value);
                     }
                    // Return 304 Not Modified and save the processing of the image
                    return  Ok((
                        StatusCode::NOT_MODIFIED,
                        cache_headers,
                        Vec::new() // Empty body
                    )); 
                }
            }
        }
    
        // If it gets here the image needs to be processed
        // Acquire a permission from the tokio semaphore
        let _permit = semaphore.acquire().await.map_err(|_| AppError::ServerBusy)?;

        let options = ProcessOptions {
            width: params.width,
            height: params.height,
            format_suffix: chosen_format.to_string(),
        };
    
        info!(id = %image_id, format = %chosen_format, "Processing image");
        
        let image_bytes = tokio::task::spawn_blocking(move || {
            image_processing::process_image(&source_path, options)
        })
        .await
        .map_err(|e| AppError:: UnexpectedError(e.into()))??;

        // Build response
        let mut response_headers = HeaderMap::new();
        response_headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static(chosen_mime)
        );
        response_headers.insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static("public, max-age=31536000, immutable")
        );
        response_headers.insert(header::VARY, HeaderValue::from_static("Accept"));
        // Intentamos crear el HeaderValue para el ETag. Si falla, simplemente no lo incluimos.
        if let Ok(etag_header_value) = HeaderValue::from_str(&etag_value) {
            response_headers.insert(header::ETAG, etag_header_value);
        }

    Ok((
        StatusCode::OK,
        response_headers,
        image_bytes
    ))
    
}