# Build Stage
FROM rust:1.75 as builder
WORKDIR /app

# Copy dependencies first to leverage Docker caching
COPY Cargo.toml Cargo.lock ./
COPY backend ./backend
COPY COPY ../../shared ./shared

RUN cargo build --release --package backend

# Runtime Stage
FROM debian:bullseye-slim
WORKDIR /app
COPY --from=builder /app/target/release/backend .

# Set production environment
ENV RUST_LOG=info

CMD ["./backend"]
