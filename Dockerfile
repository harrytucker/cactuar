FROM rust:1.72 AS chef

# Use cargo-chef to cache Rust dependency builds for Docker, it's bad the
# environment to spin your CPU that much on every Docker build!
RUN cargo install cargo-chef; \
    rustup component add rustfmt;
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12 AS cactuar

WORKDIR /app
COPY cactuar.toml cactuar.toml
COPY --from=builder --chown=root:root /app/target/release/controller /

# Expose port for tokio-console
EXPOSE 6669
CMD ["/controller"]
