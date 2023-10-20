#############
##### CHEF
FROM lukemathwalker/cargo-chef as chef

WORKDIR /usr/src/ddns

#############
##### PLANNER
FROM chef AS planner

COPY /src ./src
COPY /Cargo.toml .

RUN cargo chef prepare --recipe-path recipe.json

#############
##### BUILDER
FROM chef as builder

COPY --from=planner /usr/src/ddns/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

COPY /src ./src
COPY /Cargo.toml .

RUN cargo build --release

#############
##### RUNTIME
FROM debian:bookworm-slim AS runtime

WORKDIR /usr/local/bin

COPY --from=builder /usr/src/ddns/target/release/ddns /usr/local/bin

ENTRYPOINT ["./ddns"]