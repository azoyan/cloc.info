# Frontend build stage
FROM node:20-bookworm AS frontend-builder

WORKDIR /frontend

COPY frontend/package.json frontend/package-lock.json ./

RUN npm ci --legacy-peer-deps

COPY frontend ./

RUN npm run build

# Backend build stage
FROM rust:1.95-bookworm AS builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    git \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock build.rs ./
COPY .git ./.git

# Copy source
COPY src ./src

# Build release binary
RUN cargo build --release --locked

# Runtime stage
FROM debian:bookworm-slim

ARG APP_UID=1000
ARG APP_GID=1000

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    git \
    curl \
    libssl3 \
    wget \
    && rm -rf /var/lib/apt/lists/*

# Install scc (code counter)
RUN wget -qO- https://github.com/boyter/scc/releases/download/v3.4.0/scc_Linux_x86_64.tar.gz | tar xz \
    && mv scc /usr/local/bin/ \
    && chmod +x /usr/local/bin/scc

# Create app user matching the host UID/GID used by docker compose.
RUN groupadd --gid "${APP_GID}" appuser \
    && useradd --uid "${APP_UID}" --gid "${APP_GID}" --create-home --shell /bin/bash appuser

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/cloc /app/cloc

# Copy built frontend assets.
COPY --from=frontend-builder /frontend/dist ./dist

# Change ownership
RUN mkdir -p /app/cloc_repo \
    && chown -R appuser:appuser /app

USER appuser

EXPOSE 4000

HEALTHCHECK --interval=30s --timeout=5s --start-period=20s --retries=3 \
    CMD wget -qO- http://127.0.0.1:4000/ >/dev/null || exit 1

CMD ["/app/cloc", "0.0.0.0", "4000"]
