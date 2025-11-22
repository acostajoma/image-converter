param (
    [Parameter(Mandatory=$true)]
    [ValidateSet("build", "up", "down", "logs", "clean", "bench")]
    [string]$Command
)

$APP_NAME = "image-converter"

switch ($Command) {
    "build" {
        Write-Host "🔨 Building image..." -ForegroundColor Green
        docker build -t $APP_NAME .
    }
    "up" {
        Write-Host "🚀 Starting services..." -ForegroundColor Green
        docker compose up -d
    }
    "down" {
        Write-Host "🛑 Stopping services..." -ForegroundColor Yellow
        docker compose down
    }
    "logs" {
        docker compose logs -f
    }
    "clean" {
        Write-Host "🧹 Cleaning up..." -ForegroundColor Red
        docker compose down
        docker system prune -f
    }
    "bench" {
        Write-Host "🔥 Running benchmarks..." -ForegroundColor Cyan
        docker run --rm williamyeh/wrk `
            -t4 `
            -c20 `
            -d30s `
            -H "Accept: image/avif" `
            --timeout 3s `
            http://host.docker.internal:3000/images/test_image?width=700
    }
}