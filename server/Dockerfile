FROM rust:1.96-slim AS BUILDER
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
WORKDIR /app
ENV SQLX_OFFLINE=true

COPY Cargo.toml Cargo.lock ./
COPY .sqlx ./.sqlx
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src

COPY src ./src
COPY migrations ./migrations
RUN touch src/main.rs && cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/server /app/server
COPY --from=builder /app/migrations /app/migrations
EXPOSE 8080
CMD ["/app/server"]
