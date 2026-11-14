# The Rust toolchain and its base filesystem are immutable. Refresh the digest
# and review OS/toolchain updates together. Only apt's signed Debian repos are used.
FROM rust:1.99.1-trixie@sha256:8a4e90c7946b74626488b1446f712c0fe0ed1a596cbd3d871684bc9c5433ae07
RUN apt-get update && apt-get install -y --no-install-recommends \
    python3 clang libclang-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /workspace
