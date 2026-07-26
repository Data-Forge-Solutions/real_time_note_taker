# syntax=docker/dockerfile:1

FROM rust:1.87-bookworm AS development

WORKDIR /usr/src/rtnt

# The development stage is the shared build and CI environment.
RUN rustup component add clippy rustfmt
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
RUN cargo fetch --locked
COPY src ./src
COPY tests ./tests

FROM development AS builder
RUN cargo build --locked --release

FROM debian:bookworm-slim AS runtime

RUN groupadd --system rtnt \
    && useradd --system --gid rtnt --create-home rtnt \
    && mkdir /notes \
    && chown rtnt:rtnt /notes

COPY --from=builder /usr/src/rtnt/target/release/rtnt /usr/local/bin/rtnt

USER rtnt
WORKDIR /notes
VOLUME ["/notes"]

ENTRYPOINT ["rtnt"]
