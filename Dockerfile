# Stage 1: Builder
# We use the official Rust image as a builder.
# The 'trixie' tag corresponds to the upcoming Debian 13.
FROM rust:trixie as builder

# Create a new empty shell project
WORKDIR /usr/src/app

# Copy the dependency definitions
COPY Cargo.toml Cargo.lock ./

# Copy the source code
COPY src ./src

# Build the application in release mode
RUN cargo build --release

# Stage 2: Runner
# We use a slim Debian image for a smaller footprint.
FROM debian:trixie-slim as runner

WORKDIR /usr/src/app

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/app/target/release/image-converter .

# Expose port 3000 to the outside world
EXPOSE 3000

# Set the command to run the application
CMD ["./image-converter"]