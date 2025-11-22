use libvips::{VipsImage, ops};
use crate::error::AppError;

// Security constants: Don't process images bigger than specified dimensions
// This helps to prevent DoS attacks due to RAM overflow
const MAX_INPUT_DIMENSION: i32 = 5000;
const MAX_OUTPUT_DIMENSION: i32 = 4000;

/**
 * Struct to group transformation options
 */
#[derive(Debug)]
pub struct ProcessOptions {
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub format_suffix: String,
}

/**
    * Converts local image to specified format
    *
    * This function is blocking and CPU-intensive.
    * It must be run within a `tokio::task::spawn_blocking` call in the web handler
    * to avoid blocking the async runtime.
*/ 
pub fn process_image(source_path: &str, options: ProcessOptions) -> Result<Vec<u8>, AppError> {
    // Try to open the image
    let mut image = VipsImage::new_from_file(source_path)?;

    // Security input validation to avoid pixel bomb attacks
    let current_width = image.get_width();
    let current_height = image.get_height();

    if current_width > MAX_INPUT_DIMENSION || current_height > MAX_INPUT_DIMENSION {
        return Err(AppError::ValidationError(format!(
            "Image exceed the max allowed size of {}x{}",
            MAX_INPUT_DIMENSION, MAX_INPUT_DIMENSION
        )));
    }

    // Don't resize if the requested image is bigger than the image
    // @TODO: Return the image max width
    if let Some(target_width) = options.width {
        if target_width > MAX_OUTPUT_DIMENSION {
            return Err(AppError::ValidationError(format!(
                "Requested width ({}) exceeds the limit of {}", target_width, MAX_OUTPUT_DIMENSION
            )))
        }
        
        if target_width < current_width {
            image = ops::thumbnail_image(&image, target_width)
                .map_err(AppError::VipsError)?;
        }
    } else if let Some(target_height) = options.height {
        if target_height > MAX_OUTPUT_DIMENSION {
            return Err(AppError::ValidationError(format!(
                "Requested width ({}) exceeds the limit of {}", target_height, MAX_OUTPUT_DIMENSION
            )))
        }

        if target_height < current_height {
            let scale = (target_height as f64) / (current_height as f64);
            image = ops::resize(&image, scale)
                .map_err(AppError::VipsError)?;
        }
    }

    // Smart compression parameters
    let format_suffix = options.format_suffix.as_str();
    let options_string = match format_suffix {
        // AVIF is slow to compress 'speed=6' is a good balance (0=slow/best, 9=fast/worst)
        // Q=50 on AVIF tends to be visually equal to JPG Q=80 pero but with 1/2 of the file size.
        ".avif" => format!("{}[Q=50,speed=6]", format_suffix),
        
        // WebP: Faster to compress. Q=75 is Google's standard.
        ".webp" => format!("{}[Q=75]", format_suffix),

        // Any other suffix uses jpg
        _ => format_suffix.to_string(),
    };

    // Execute the conversion
    let buffer = image.image_write_to_buffer(&options_string)
        .map_err(AppError::VipsError)?;

    Ok(buffer)
}