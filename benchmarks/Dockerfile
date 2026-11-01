# The Rust toolchain and its base filesystem are immutable. Refresh the digest
# and review OS/toolchain updates together. Only apt's signed Debian repos are used.
FROM rust:1.99.1-trixie@sha256:09027f873d45b8ecd1dc86f1fa13b7a62b875c961f017a0e331267b350430137
RUN apt-get update && apt-get install -y --no-install-recommends \
    python3 clang libclang-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /workspace
