FROM rust:1.88-bookworm AS rust

FROM rust AS builder
RUN apt-get update && apt-get -y --no-install-recommends install \
    protobuf-compiler \
    libprotobuf-dev \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
ADD Cargo.toml .
ADD Cargo.lock .
ADD build.rs .
ADD proto proto
ADD src src
RUN cargo build --release

FROM rust AS spray
WORKDIR /app
COPY --from=builder /app/target/release/sqd-spray .
ENTRYPOINT ["/app/sqd-spray"]