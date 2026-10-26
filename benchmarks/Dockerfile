# The Rust toolchain and its base filesystem are immutable. Refresh the digest
# and review OS/toolchain updates together. Only apt's signed Debian repos are used.
FROM rust:1.97.1-trixie@sha256:b1b3c9c0d921d7fa0a6d1f9ec7e4eab87f8c8ec97644c3d791450f131dec813f
RUN apt-get update && apt-get install -y --no-install-recommends \
    python3 clang libclang-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /workspace
