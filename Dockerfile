FROM rust:latest AS builder

ARG TARGET
RUN rustup target add ${TARGET}
RUN apt-get update && apt-get install -y \
    gcc-mingw-w64-x86-64 \
    gcc-aarch64-linux-gnu \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .
RUN cargo build --release --target ${TARGET}

FROM scratch AS export
ARG TARGET
COPY --from=builder /app/target/${TARGET}/release/mermaid-ascii* /
