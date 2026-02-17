FROM ghcr.io/astral-sh/uv:python3.12-bookworm-slim AS builder

WORKDIR /app
COPY . .
RUN uv build

FROM scratch AS export
COPY --from=builder /app/dist/* /
