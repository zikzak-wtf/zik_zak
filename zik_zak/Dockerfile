FROM rust:1.78-slim AS builder

WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    xz-utils \
    libclang-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy Cargo files
COPY Cargo.toml ./

# Copy source code
COPY src/ src/

# Build the application
RUN cargo build --release --bin pure-accounting-server

# Final stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    curl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/pure-accounting-server .

# Copy any other necessary files
COPY recipes.json .

# Expose port
EXPOSE 3003

# Start the server
CMD ["./pure-accounting-server"]
