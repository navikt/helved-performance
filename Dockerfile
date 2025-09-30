FROM rust:1.90-slim AS base
RUN apt-get update \
    && apt-get install -y pkg-config musl-tools libssl-dev build-essential \
    && rustup target add x86_64-unknown-linux-musl \
    && rm -rf /var/lib/apt/lists/*
RUN cargo install sccache

WORKDIR /app
ENV SCCACHE_DIR=/sccache
ENV RUSTC_WRAPPER=sccache

COPY sccache /sccache

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main(){}" > src/main.rs \
    && cargo build --release --target x86_64-unknown-linux-musl \
    && rm -rf src

COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl --bin helved-performance

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=base /app/target/x86_64-unknown-linux-musl/release/helved-performance /usr/local/bin

ENTRYPOINT ["/usr/local/bin/helved-performance"]

