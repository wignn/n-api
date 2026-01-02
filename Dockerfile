FROM rust:alpine AS builder

LABEL authors="tigfi"

RUN apk add --no-cache \
    musl-dev \
    pkgconfig \
    openssl-dev \
    openssl-libs-static \
    postgresql-dev

WORKDIR /app

COPY Cargo.toml ./
COPY Cargo.lock ./

RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm src/main.rs

COPY src ./src
RUN touch src/main.rs && cargo build --release

FROM alpine:latest

RUN apk add --no-cache \
    ca-certificates \
    libssl3 \
    libpq

RUN adduser -D -h /app appuser

WORKDIR /app

COPY --from=builder /app/target/release/novel-api /app/novel-api

RUN chown -R appuser:appuser /app

USER appuser

EXPOSE 4001

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:4001/healthy || exit 1

CMD ["./novel-api"]