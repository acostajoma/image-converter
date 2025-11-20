use std::{
    io::Cursor,
};
use image::{ImageFormat, ImageReader};
use serde::Deserialize;
use axum::{
    Router,
    extract::{Path, Query, rejection::QueryRejection},
    http::{StatusCode, header::{self, HeaderMap, HeaderValue}},
    routing::get,
    response::IntoResponse
};

fn convert_image(image_name: &String, format: ImageFormat) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // 1. Load the image from a file.
    let img = ImageReader::open(image_name)?.decode()?;
    // 2. Create an in-memory buffer.
    let mut bytes: Vec<u8> = Vec::new();
    // 3. Write the image to the buffer in the chosen format.
    img.write_to(&mut Cursor::new(&mut bytes), format)?;
    Ok(bytes)
}


#[derive(Debug, Deserialize)]
struct Size {
    width: Option<u16>,
    height: Option<u16>,
}

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Image converter" }))
        .route("/images/{query}", get(handler))
        .fallback(fallback);

    
    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// 1. `PREFERRED_FORMATS` ahora es `static` para que exista una sola instancia en memoria.
static PREFERRED_FORMATS: &[(&str, ImageFormat)] = &[
    ("image/avif", ImageFormat::Avif),
    ("image/webp", ImageFormat::WebP),
    ("image/jpeg", ImageFormat::Jpeg),
    ("image/png", ImageFormat::Png),
];

// Función de utilidad para obtener el MIME type desde ImageFormat
fn get_mime_from_format(format: ImageFormat) -> &'static str {
    match format {
        ImageFormat::Avif => "image/avif",
        ImageFormat::WebP => "image/webp",
        ImageFormat::Jpeg => "image/jpeg",
        ImageFormat::Png => "image/png",
        _ => "application/octet-stream", // Un tipo genérico si no hay coincidencia
    }
}

#[axum::debug_handler]
async fn handler(
    Path(path): Path<String>,
    _query_result: Result<Query<Size>, QueryRejection>, // Lo mantenemos por si se usa en el futuro
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // 2. Establece un formato por defecto (Jpeg) por si no hay coincidencias.
    let mut chosen_format = ImageFormat::Jpeg;

    // 3. Revisa el header "Accept" del cliente.
    if let Some(accept_header) = headers.get(header::ACCEPT).and_then(|val| val.to_str().ok()) {
        // 4. Busca la mejor coincidencia entre las preferencias del servidor y las del cliente.
        for (mime, format) in PREFERRED_FORMATS.iter() {
            if accept_header.contains(mime) {
                chosen_format = *format;
                break; // Encontramos el mejor formato soportado, salimos del bucle.
            }
        }
    }

    // Llama a la función para convertir la imagen y obtener los bytes.
    match convert_image(&path, chosen_format) {
        Ok(image_bytes) => {
            let mime_type = get_mime_from_format(chosen_format);
            let mut headers = HeaderMap::new();
            headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(mime_type));

            // Devuelve los headers y el cuerpo de la imagen.
            Ok((headers, image_bytes))
        },
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Error al procesar la imagen: {}", e))),
    }
}

async fn fallback() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "Not Found")
}
