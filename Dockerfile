# Stage 1: Build Svelte 5 UI
FROM node:22-bookworm-slim AS ui-builder
WORKDIR /app/ui
COPY ui/package*.json ./
RUN npm install
COPY ui ./
RUN npm run build

# Stage 2: Build Rust backend
FROM rust:1-bookworm AS builder
WORKDIR /src
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates crates
ENV CARGO_PROFILE_RELEASE_DEBUG=0
ENV CARGO_PROFILE_RELEASE_STRIP=true
ENV CARGO_INCREMENTAL=0
RUN cargo build --release --locked -p nodlance-api

# Stage 3: Minimal secure runtime
FROM debian:bookworm-slim AS runtime
RUN useradd --system --uid 10001 --create-home nodlance
WORKDIR /app
COPY --from=builder /etc/ssl/certs /etc/ssl/certs
COPY --from=builder /src/target/release/nodlance-api /usr/local/bin/nodlance-api
COPY --from=ui-builder /app/ui/dist /app/ui/dist
USER 10001:10001
ENV PORT=8080 RUST_LOG=info NODLANCE_STATIC_DIR=/app/ui/dist
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/nodlance-api"]
