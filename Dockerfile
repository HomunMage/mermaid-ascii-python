# Stage 1: Install dependencies using uv
FROM ghcr.io/astral-sh/uv:python3.12-bookworm-slim AS builder

WORKDIR /app

# Install dependencies first (cached layer)
COPY pyproject.toml uv.lock ./
RUN uv sync --frozen --no-dev --no-install-project

# Copy source and install the project itself
COPY src/ src/
RUN uv sync --frozen --no-dev

# Stage 2: Minimal runtime image
FROM python:3.12-slim-bookworm AS runtime

# Create non-root user
RUN groupadd -g 1000 mermaid && \
    useradd -u 1000 -g mermaid -m mermaid

WORKDIR /app

# Copy the virtual environment from builder
COPY --from=builder /app/.venv /app/.venv

# Copy the source package
COPY --from=builder /app/src /app/src

# Use the venv Python directly (no uv needed at runtime)
ENV PATH="/app/.venv/bin:$PATH"

USER mermaid

ENTRYPOINT ["python", "-m", "mermaid_ascii"]
CMD ["--help"]
