FROM rust:1 AS chef 
RUN cargo install cargo-chef 
WORKDIR /semafor

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
COPY --from=planner /semafor/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin semafor

# We do not need the Rust toolchain to run the binary!
FROM debian:bookworm-slim AS runtime
WORKDIR /semafor
# hadolint ignore=DL3008
RUN apt-get update && apt-get install --no-install-recommends -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /semafor/target/release/semafor /usr/local/bin
ENTRYPOINT ["/usr/local/bin/semafor"]