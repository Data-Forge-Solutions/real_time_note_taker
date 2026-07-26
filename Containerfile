# syntax=docker/dockerfile:1

FROM rust:1.87-bookworm AS builder

WORKDIR /usr/src/rtnt

# Fetch dependencies separately so source-only changes can reuse this layer.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src \
    && printf 'fn main() {}\n' > src/main.rs \
    && cargo build --locked --release \
    && rm -rf src target/release/deps/real_time_note_taker* \
        target/release/deps/rtnt* target/release/rtnt

COPY src ./src
COPY tests ./tests
RUN cargo build --locked --release

FROM builder AS test
RUN rustup component add clippy
RUN cargo test --locked --all-targets \
    && cargo test --locked --doc \
    && cargo clippy --locked --all-targets --all-features -- -D warnings -W clippy::pedantic

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
