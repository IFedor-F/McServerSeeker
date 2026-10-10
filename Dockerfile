FROM rust:alpine AS chef
WORKDIR /workspace
RUN apk add --no-cache musl-dev pkgconfig openssl-dev openssl-libs-static protoc protobuf-dev
RUN cargo install cargo-chef

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /workspace/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo build --release -p mc_server_seeker_worker
RUN cargo build --release -p mc_server_seeker_manager

FROM alpine:latest AS mc_server_seeker_worker
WORKDIR /app
RUN apk add --no-cache masscan ca-certificates libgcc
COPY --from=builder /workspace/target/release/mc_server_seeker_worker ./mc_server_seeker_worker
CMD ["./mc_server_seeker_worker"]


FROM alpine:latest AS mc_server_seeker_manager
WORKDIR /app
RUN apk add --no-cache ca-certificates libgcc
COPY --from=builder /workspace/target/release/mc_server_seeker_manager ./mc_server_seeker_manager
CMD ["./mc_server_seeker_manager"]
