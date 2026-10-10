# ---- Build stage ----
FROM rust:1-bookworm AS builder

# cargo-leptos drives the dual (server + wasm) build
RUN rustup target add wasm32-unknown-unknown \
    && curl -L --proto '=https' --tlsv1.2 -sSf \
       https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash \
    && cargo binstall -y cargo-leptos

WORKDIR /app
COPY app/ .

RUN cargo leptos build --release -vv

# CLI for registration codes. Same flags as cargo-leptos' server build, so the
# compiled dependencies are reused.
RUN cargo build --release --no-default-features --features ssr --bin regcode

# ---- Runtime stage ----
FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/owesome /app/owesome
COPY --from=builder /app/target/release/regcode /app/regcode
COPY --from=builder /app/target/site /app/site
COPY --from=builder /app/Cargo.toml /app/Cargo.toml

ENV RUST_LOG="info" \
    LEPTOS_OUTPUT_NAME="owesome" \
    LEPTOS_SITE_ROOT="site" \
    LEPTOS_SITE_PKG_DIR="pkg" \
    LEPTOS_SITE_ADDR="0.0.0.0:8080" \
    LEPTOS_RELOAD_PORT="3001" \
    LEPTOS_ENV="PROD"

EXPOSE 8080

CMD ["/app/owesome"]
