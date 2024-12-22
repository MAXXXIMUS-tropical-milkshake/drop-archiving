
FROM rust:1.82-slim as builder

WORKDIR /app

RUN apt-get update && apt-get install -y \
    build-essential \
    libssl-dev \
    pkg-config \
    libpq-dev \
    ffmpeg \
    libavutil-dev \
    libavcodec-dev \
    libavformat-dev \
    libswscale-dev \
    libavfilter-dev \
    libavdevice-dev \
    libprotobuf-dev \
    protobuf-compiler \
    clang \
    llvm-dev \
    git \
    libclang-dev \
    curl \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

ENV PKG_CONFIG_PATH=/usr/lib/x86_64-linux-gnu/pkgconfig

RUN cargo install sqlx-cli --no-default-features --features postgres


RUN git clone https://github.com/googleapis/api-common-protos /app/proto/api-common-protos
COPY ./Cargo.toml ./Cargo.lock ./
COPY ./proto /app/proto
COPY ./build.rs ./
COPY src ./src

COPY ./migrations /app/migrations


RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && apt-get install -y \
    libssl3 \
    libpq5 \
    ffmpeg \
    ca-certificates \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/cargo/bin/sqlx /usr/local/bin/sqlx
COPY --from=builder /app/target/release/drop /app/drop
COPY --from=builder /app/migrations /app/migrations

EXPOSE 3000

CMD ["sh", "-c", "sqlx migrate run --database-url $DATABASE_URL && /app/drop"]
