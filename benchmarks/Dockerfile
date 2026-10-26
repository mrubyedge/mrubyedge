# The Rust toolchain and its base filesystem are immutable. Refresh the digest
# and review OS/toolchain updates together. Only apt's signed Debian repos are used.
FROM rust:1.97.1-bookworm@sha256:0e2bcaef56d041a486784e54104a81aebe0da44bd03019bd70bc0401e42e4a97
RUN apt-get update && apt-get install -y --no-install-recommends \
    python3 clang libclang-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /workspace
