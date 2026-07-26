FROM rust:1.85-slim AS builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/mergemint-backend /app/mergemint-backend

EXPOSE 8080

CMD ["/app/mergemint-backend"]
