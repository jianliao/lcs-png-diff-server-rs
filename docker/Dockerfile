# Rust as the base image
FROM rust:latest as builder

# Create a new empty shell project
RUN USER=root cargo new --bin lcs-png-diff-server
WORKDIR /lcs-png-diff-server

# Copy our manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# Build only the dependencies to cache them
RUN cargo build --release
RUN rm src/*.rs

# Copy the source code
COPY ./src ./src

# Build for release.
RUN rm ./target/release/deps/lcs_png_diff_server*
RUN cargo build --release

# The final base image
FROM debian:buster-slim

RUN apt-get update && \
  apt-get install -y libssl-dev && \
  rm -rf /var/lib/apt/lists/*

# Copy from the previous build
COPY --from=builder /lcs-png-diff-server/target/release/lcs-png-diff-server /usr/src/lcs-png-diff-server

WORKDIR /usr/src

VOLUME \
  /usr/src/assets

# Expose the binary
ENTRYPOINT ["/usr/src/lcs-png-diff-server"]
