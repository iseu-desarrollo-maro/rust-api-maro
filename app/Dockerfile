FROM rust:1.88-slim-bookworm AS builder
RUN apt-get update && apt-get install -y pkg-config libssl-dev build-essential
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 ca-certificates tzdata && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/app/target/release/mi-proyecto-enpoint /usr/local/bin/mi-proyecto-enpoint
EXPOSE 7001 # Puerto que escucha la aplicación Rust
CMD ["/usr/local/bin/mi-proyecto-enpoint"]