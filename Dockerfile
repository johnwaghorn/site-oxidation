FROM node:24-alpine AS frontend
ARG VITE_API_KEY
ENV VITE_API_KEY=$VITE_API_KEY
WORKDIR /frontend
COPY frontend/package*.json ./
RUN npm ci
COPY frontend/ ./
RUN npm run build

FROM rust:1.92 AS backend
WORKDIR /app
COPY Cargo.* ./
COPY src/ ./src/
COPY migrations/ ./migrations/
RUN cargo build --release

FROM debian:trixie-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=backend /app/target/release/site-oxidation .
COPY --from=frontend /static ./static
COPY migrations/ ./migrations/
EXPOSE 8080
CMD ["./site-oxidation"]
