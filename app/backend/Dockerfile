FROM lukemathwalker/cargo-chef:latest-rust-1.97.1-alpine3.24 AS chef
WORKDIR /app

FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release

FROM rust:1.97.1-slim-bookworm AS runtime
WORKDIR /app
RUN rustup target add wasm32-wasip2
COPY --from=builder /app/target/release/backend /usr/local/bin
ENTRYPOINT ["/usr/local/bin/backend"]
