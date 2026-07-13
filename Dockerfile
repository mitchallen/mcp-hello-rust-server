# --- Stage 1: Build ---
# Build a fully-static musl binary on Alpine's Rust image. Alpine's default host
# target is *-unknown-linux-musl, whose CRT is statically linked, so the output
# has no dynamic libc dependency and can run on an empty base. Building here (not
# on the runtime base) keeps the toolchain out of the shipped image entirely.
FROM rust:1-alpine AS builder
WORKDIR /app

# musl-dev provides the C runtime/linker bits Rust needs to link against musl.
RUN apk add --no-cache musl-dev

# Build dependencies first as a cached layer, using only the manifests plus a
# stub crate. When only src/ changes, this expensive layer is reused.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src \
    && echo 'fn main() {}' > src/main.rs \
    && echo '' > src/lib.rs \
    && cargo build --release \
    && rm -rf src

# Now build the real sources. Only our own crate recompiles; deps are cached.
# `touch` bumps the sources' mtime above the stub artifacts' — without it cargo
# sees the COPYed files as older than the cached build and skips recompiling,
# shipping the empty stub binary.
COPY src ./src
RUN touch src/main.rs src/lib.rs \
    && cargo build --release \
    && cp target/release/mcp-hello-rust-server /mcp-hello-rust-server

# --- Stage 2: Production ---
# Distroless Chainguard/Wolfi "static" base — no shell, no package manager, and
# it already runs as the non-root 'nonroot' user (uid 65532). It carries only CA
# certs and tzdata, so a static binary on top scans near-zero vulnerabilities
# (nothing for a Debian-style OS-package CVE to attach to).
FROM cgr.dev/chainguard/static:latest AS prod

# Serve over streamable HTTP by default so the container is reachable on a
# published port; MCP_TRANSPORT=stdio switches to stdio for client-launched use.
ENV MCP_TRANSPORT=http \
    HOST=0.0.0.0 \
    PORT=8000

COPY --from=builder /mcp-hello-rust-server /usr/local/bin/mcp-hello-rust-server

EXPOSE 8000
ENTRYPOINT ["/usr/local/bin/mcp-hello-rust-server"]
