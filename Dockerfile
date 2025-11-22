# ==============================================================================
# STAGE 1: CHEF BASE (OS + System Tools)
# ==============================================================================
FROM rust:alpine3.20 AS chef

WORKDIR /app

# Instalamos herramientas de compilación.
# clang y clang-libclang son requeridos por bindgen.
# musl-dev y build-base para el compilador de C.
RUN apk add --no-cache \
    build-base \
    musl-dev \
    vips-dev \
    pkgconf \
    clang \
    clang-libclang

RUN cargo install cargo-chef
ENV RUSTFLAGS="-C target-feature=-crt-static"

# ==============================================================================
# STAGE 2: PLANNER 
# ==============================================================================
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ==============================================================================
# STAGE 3: BUILDER (Compila Dependencias y luego la App)
# ==============================================================================
FROM chef as builder

# Dependency Cache
COPY --from=planner /app/recipe.json recipe.json
# "Cocina" las dependencias. Si tu Cargo.toml no cambió, Docker usa el caché de esta capa
# y se salta la compilación de las ~200 librerías (ahorrando 5-10 minutos).
# Avoid recompiling libraries if cargo.toml did not change
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release

# ==============================================================================
# STAGE 4: RUNNER
# ==============================================================================
FROM alpine:3.20 AS runner

WORKDIR /app
RUN addgroup -S appgroup && adduser -S appuser -G appgroup

# Instalamos las librerías runtime de Vips
RUN apk add --no-cache \
    vips \
    libheif \
    aom-libs \
    ca-certificates \
    libgcc \
    libimagequant

# Copy the binary from the builder
COPY --from=builder /app/target/release/image-converter /usr/local/bin/

# Copy static files for the testing
COPY images/ ./images/
RUN chown -R appuser:appgroup /app


EXPOSE 3000
ENV RUST_LOG=info

USER appuser

CMD ["image-converter"]