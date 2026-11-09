FROM rust:1-slim-bookworm AS build
WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY src src
COPY migrations migrations
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=build /app/target/release/{{project-name}} /usr/local/bin/app
ENV HTTP_HOST=0.0.0.0
EXPOSE 3001
CMD ["app"]
