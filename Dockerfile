# -------- Stage 1: Build --------
FROM rust:1.87 as build-deps

# Set working directory
WORKDIR /app

# Copy Cargo files and build dummy project to cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release || true

# Copy the full source code
COPY . .

# Build the actual binary
RUN cargo build --release --bin bpp-onest-lite

# -------- Stage 2: Runtime --------
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libpq5 \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy the built binary and config
COPY --from=build-deps /app/target/release/bpp-onest-lite /app/bpp-onest-lite
COPY config ./config

# Expose the port
EXPOSE 3009

# Run the binary
CMD ["./bpp-onest-lite"]
