FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
rustup target add x86_64-unknown-linux-musl
cargo build --release 

COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --target=x86_64-unknown-linux-musl --bin helved-performance

FROM debian:bookworm-slim AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/helved-performance /usr/local/bin
ENTRYPOINT ["/usr/local/bin/helved-performance"]
