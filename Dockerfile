FROM rust:1.94-slim AS builder

RUN apt-get update && apt-get install -y libsqlite3-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
COPY templates ./templates
COPY static ./static

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y libsqlite3-0 && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/rust-htmx-template ./rust-htmx-template
RUN mkdir /data

ENV HOST=0.0.0.0 PORT=3000 DATABASE_URL=/data/db.sqlite

EXPOSE 3000

CMD ["./rust-htmx-template"]

