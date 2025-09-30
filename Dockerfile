FROM rust:1.90-alpine3.22 AS builder
WORKDIR /app
RUN apk add --no-cache \
    musl-dev \
    gcc \
    g++ \
    openssl-dev \
    pkgconfig \
    bash

ENV RUSTFLAGS="-C target-feature=+crt-static"
COPY Cargo.toml Cargo.lock ./
RUN mkdir src \
    && echo "fn main(){}" > src/main.rs \
    && cargo build --release --target x86_64-unknown-linux-musl \
    && rm -rf src
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM alpine:3.19 AS final
WORKDIR /app
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/helved-performance /usr/local/bin/helved-performance
CMD ["/usr/local/bin/helved-performance"]
