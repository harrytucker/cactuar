FROM rust:1.67-bullseye AS chef

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

FROM gcr.io/distroless/cc AS cactuar

WORKDIR /app
COPY cactuar.toml cactuar.toml
COPY --from=builder --chown=root:root /app/target/release/controller /usr/local/bin/

CMD ["controller"]
