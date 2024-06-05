# syntax=docker/dockerfile:1

################################################################################
# Build stage

ARG RUST_VERSION=1.78.0
ARG APP_NAME=klotski_solver

FROM rust:${RUST_VERSION}-buster AS build
ARG APP_NAME
WORKDIR /app

RUN apt-get update && apt-get install -y \
    clang \
    lld \
    pkg-config \
    libpq-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    --mount=type=bind,source=migrations,target=migrations \
cargo build --locked --release && \
cp ./target/release/$APP_NAME /bin/server

################################################################################
# Final stage

FROM debian:buster-slim AS final

ARG ENVIRONMENT
ARG LOG_LEVEL
ARG BIND_URL
ARG BIND_PORT
ARG PG_PORT
ARG PG_DATABASE
ARG PG_USERNAME
ARG PG_PASSWORD
ARG SENTRY_DSN

ENV ENVIRONMENT=${ENVIRONMENT}
ENV LOG_LEVEL=${LOG_LEVEL}
ENV BIND_URL=${BIND_URL}
ENV BIND_PORT=${BIND_PORT}
ENV PG_PORT=${PG_PORT}
ENV PG_DATABASE=${PG_DATABASE}
ENV PG_USERNAME=${PG_USERNAME}
ENV PG_PASSWORD=${PG_PASSWORD}
ENV SENTRY_DSN=${SENTRY_DSN}

ARG UID=10001

RUN apt-get update && apt-get install -y libpq5 && \
    adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser && \
    rm -rf /var/lib/apt/lists/*

USER appuser

COPY --from=build /bin/server /bin/

EXPOSE 8080

CMD ["/bin/server"]
