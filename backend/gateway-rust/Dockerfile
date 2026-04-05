FROM rust:1.89-alpine AS builder

RUN apk add --no-cache musl-dev pkgconfig openssl-dev
WORKDIR /app

COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
COPY migrations migrations
COPY src src

RUN cargo build --release

FROM alpine:3.19

RUN addgroup -S velium && adduser -S velium -G velium
RUN apk add --no-cache ca-certificates

WORKDIR /app
COPY --from=builder /app/target/release/velium-gateway /usr/local/bin/velium-gateway

USER velium
EXPOSE 8080

HEALTHCHECK --interval=15s --timeout=3s --start-period=10s --retries=3 \
  CMD wget -q -O /dev/null http://localhost:8080/health || exit 1

ENTRYPOINT ["velium-gateway"]
