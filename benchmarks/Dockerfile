# The Rust toolchain and its base filesystem are immutable. Refresh the digest
# and review OS/toolchain updates together. Only apt's signed Debian repos are used.
FROM rust:1.100.0-trixie@sha256:223cc32ddca8f3ee90c436dbc94b6b5f052ea7f9653c5201e9612f08ad7d21b8
RUN apt-get update && apt-get install -y --no-install-recommends \
    python3 clang libclang-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /workspace
