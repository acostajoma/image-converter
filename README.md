# Delete old images
docker system prune -f
# Stop and remove the existing container if it's running to avoid name conflicts
docker rm -f image-converter
# Build the docker Image
docker build -t image-converter .
# Execute container locally with restricted resources. Equally to the resources it would have on the cloud.
docker run -p 3000:3000 --cpus="2.0" --name image-converter image-converter

# Execute test of how many request can the container attend
docker run --rm williamyeh/wrk -t4 -c20 -d15s -H "Accept: image/webp" http://host.docker.internal:3000/images/Sheriff_labrador2

# 🚀 High-Performance Image Optimization Service

A robust, enterprise-grade microservice built with **Rust (Axum)** and **Libvips**. Designed for high concurrency, low memory footprint, and efficient image format conversion (AVIF, WebP, PNG, JPG).

## ⚡ Key Features

* **Smart Resizing:** Uses `libvips` with "shrink-on-load" for minimal memory usage.
* **Format Negotiation:** Automatically serves AVIF or WebP based on the `Accept` header.
* **Smart Caching:** Implements strong ETag generation based on file metadata and processing options (304 Not Modified support).
* **Memory Safety:** Uses `jemalloc` to prevent memory fragmentation under heavy load.
* **Concurrency Control:** Semaphore-based limiting to prevent CPU saturation.
* **Security:** Request validation to prevent "Pixel Bomb" attacks.

---

## ⚙️ Configuration

The application is configured via environment variables. Create a `.env` file in the root directory:

| Variable | Default | Description |
| :--- | :--- | :--- |
| `HOST` | `0.0.0.0` | Interface to bind to (0.0.0.0 for Docker). |
| `PORT` | `3000` | Port to listen on. |
| `IMAGES_DIR` | `images` | Local directory containing source images. |
| `MAX_CONCURRENCY` | *CPUs* | Max concurrent image conversions. Match this to your CPU limit. |
| `VIPS_CACHE_MB` | `128` | RAM allocated for Libvips operation cache (MB). |
| `RUST_LOG` | `info` | Logging level (`debug`, `info`, `warn`, `error`). |

**Example `.env`:**
```ini
HOST=0.0.0.0
PORT=3000
IMAGES_DIR=images
MAX_CONCURRENCY=4
VIPS_CACHE_MB=256
RUST_LOG=image_converter=info,tower_http=info

## 🛠️ Quick Start

To start the service locally with all configurations applied:

For Linux/Unix based Systems
```bash
make build
make up

For Windows
```powershell
.\manage.ps1 build
.\manage.ps1 up

View Makefile and manage.ps1 scripts for more commands