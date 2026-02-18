# Multi-target Rust build via Docker Buildx
FROM rust:1.85-slim-bookworm AS builder

ARG TARGET=x86_64-unknown-linux-gnu

WORKDIR /app

RUN apt-get update \
    && apt-get install -y gcc-aarch64-linux-gnu gcc-mingw-w64-x86-64 \
    && rm -rf /var/lib/apt/lists/* \
    && rustup target add ${TARGET}

COPY Cargo.toml Cargo.lock ./
COPY src/rust/ src/rust/

ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
ENV CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc
RUN cargo build --release --target ${TARGET}

# Export binary via scratch image
FROM scratch AS export
ARG TARGET=x86_64-unknown-linux-gnu
COPY --from=builder /app/target/${TARGET}/release/mermaid-ascii* /
