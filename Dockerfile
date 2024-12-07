FROM lukemathwalker/cargo-chef:latest as chef
ARG RUST_VERSION=1.82.0
ARG APP_NAME=traefik-multi-host-mapper
WORKDIR /app

FROM chef AS planner
COPY ./Cargo.toml ./Cargo.lock ./
COPY ./src ./src
RUN cargo chef prepare

FROM chef AS builder
COPY --from=planner /app/recipe.json .
RUN cargo chef cook --release
COPY . .
RUN cargo build --release
RUN mv ./target/release/$APP_NAME ./app

FROM debian:stable-slim AS runtime
WORKDIR /app
# Install OpenSSL 3 and other runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
  openssl \
  ca-certificates && \
  rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/app /usr/local/bin/
COPY ./config.toml /app/
ENTRYPOINT ["/usr/local/bin/app"]
