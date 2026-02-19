# Multi-stage build for ReMem

FROM rust:1.93 AS builder 

WORKDIR /app

# Copy manifest
COPY Cargo.toml Cargo.lock* ./

# Copy source
COPY src ./src

# Build
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/re_mem .

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

EXPOSE 3000

ENV RUST_LOG=info

CMD ["./re_mem"]
