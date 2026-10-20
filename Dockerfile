FROM rust:1-bookworm AS builder
WORKDIR /src
COPY Cargo.toml Cargo.toml
COPY rust-toolchain.toml rust-toolchain.toml
COPY crates crates
RUN cargo build --release -p nodlance-api

FROM debian:bookworm-slim AS runtime
RUN useradd --system --uid 10001 --create-home nodlance \
    && apt-get update \
    && apt-get install --yes --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /src/target/release/nodlance-api /usr/local/bin/nodlance-api
USER 10001:10001
ENV PORT=8080 RUST_LOG=info
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/nodlance-api"]
