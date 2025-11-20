use std::{
    io::Cursor,
    collections::HashMap
};
use image::{ImageFormat, ImageReader};
use std::fs::File;
use std::io::Write;
// fn main() {
//     convert_image();
// }
use serde::Deserialize;

// fn convert_image()-> Result<(), Box<dyn std::error::Error>> {
//     // 1. Load the image from a file.
//     let img = ImageReader::open("my-image.png")?.decode()?;
//     // 2. Create an in-memory buffer.
//     let mut bytes: Vec<u8> = Vec::new();
//     // 3. Write the image to the buffer in JPEG format.
//     img.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Jpeg)?;
//     // 4. Save the buffer to a new file.
//     let mut file = File::create("new-image.jpg")?;
//     file.write_all(&bytes)?;
//     Ok(())
// }

use axum::{
    Router, extract::{Path, Query, rejection::QueryRejection}, http::{StatusCode, header::{self, HeaderMap}}, routing::get
};


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

#[axum::debug_handler]
async fn handler(
    Path(path): Path<String>,
    query_result: Result<Query<Size>, QueryRejection>,
    headers: HeaderMap,
) -> Result<impl axum::response::IntoResponse, (StatusCode, String)> {
    // 2. Establece un formato por defecto (ej. Jpeg) por si no hay coincidencias.
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

    match query_result {
        Ok(Query(size)) => {
            // Usamos format! para crear un nuevo String con ambos datos.
            // {:?} es el "formateador de depuración", que imprimirá el enum ImageFormat de forma legible.
            let response_string = format!("Ruta solicitada: {}\nFormato elegido: {:?}", path, chosen_format);
            println!("{:#?}", size); // Mantenemos el log para la consola
            Ok(response_string)
        }
        Err(rejection) => Err((StatusCode::BAD_REQUEST, format!("Error en los parámetros de la consulta: {}", rejection))),
    }
}
async fn fallback() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "Not Found")
}
