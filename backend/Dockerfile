FROM rust:1-bookworm AS builder

WORKDIR /api

COPY Cargo.toml Cargo.lock ./
COPY migration/Cargo.toml ./migration/Cargo.toml

RUN mkdir -p src migration/src && \
    echo "fn main() {}" > src/main.rs && \
    echo "" > migration/src/lib.rs && \
    cargo build --release && \
    rm -rf src migration/src

COPY src ./src
COPY migration/src ./migration/src
RUN touch src/main.rs migration/src/lib.rs && cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /api/target/release/backend ./backend

EXPOSE 3000

CMD ["./backend"]