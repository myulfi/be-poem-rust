FROM rust:latest AS builder
WORKDIR /usr/src/app

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release || true  # build dummy untuk cache

COPY . .

RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /usr/src/app

RUN apt-get update && apt-get install -y libpq5 ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/be-poem-rust .

CMD ["./be-poem-rust"]