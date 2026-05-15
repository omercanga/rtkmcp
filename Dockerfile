# Multi-stage build — minimal runtime image (~5MB)
FROM rust:1.82-alpine AS builder

WORKDIR /build
RUN apk add --no-cache musl-dev

COPY Cargo.toml Cargo.lock ./
# Cache deps layer
RUN mkdir src && echo "fn main(){}" > src/main.rs && cargo build --release
RUN rm -rf src

COPY src ./src
# Force rebuild of rtkmcp only
RUN touch src/main.rs && cargo build --release

# ── Runtime image ─────────────────────────────────────────────────────────────
FROM alpine:3.20

RUN apk add --no-cache git ripgrep

COPY --from=builder /build/target/release/rtkmcp /usr/local/bin/rtkmcp

WORKDIR /workspace

ENTRYPOINT ["rtkmcp"]
