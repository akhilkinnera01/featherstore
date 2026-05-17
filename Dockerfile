FROM rust:1.81-bookworm AS builder
WORKDIR /app
COPY Cargo.toml ./
COPY crates ./crates
RUN cargo build --release -p featherstore-server

FROM debian:bookworm-slim AS runtime
RUN useradd --system --uid 10001 --home /nonexistent --shell /usr/sbin/nologin featherstore \
    && apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/featherstore-server /usr/local/bin/featherstore-server
USER 10001:10001
EXPOSE 8080
ENV FEATHERSTORE_BIND_ADDR=0.0.0.0:8080 FEATHERSTORE_LOG_FORMAT=json
ENTRYPOINT ["/usr/local/bin/featherstore-server"]
