FROM node:24-alpine AS frontend
WORKDIR /frontend
COPY frontend/package*.json ./
RUN npm ci
COPY frontend/ ./
RUN npm run build

FROM rust:1.95 AS backend
WORKDIR /app
COPY Cargo.* ./
COPY src/ ./src/
COPY migrations/ ./migrations/
RUN cargo build --release

FROM debian:trixie-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl tzdata && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=backend /app/target/release/site-oxidation .
COPY --from=frontend /static ./static
COPY migrations/ ./migrations/
# Unraid's nobody:users (99:100) so container can write the SQLite DB & session key
RUN mkdir -p /app/data && chown 99:100 /app/data
# Drop root: run as the uid:gid that owns /app/data and Unraid's appdata share
USER 99:100
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
  CMD curl -fsS http://localhost:8080/api/health || exit 1
CMD ["./site-oxidation"]
