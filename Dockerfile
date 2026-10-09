FROM rust:alpine AS base
WORKDIR /workspace
COPY . .

FROM base AS builder-mc_server_seeker_worker
RUN apk add --no-cache musl-dev pkgconfig openssl-dev openssl-libs-static protoc protobuf-dev
RUN cargo build --release -p mc_server_seeker_worker

FROM base AS builder-mc_server_seeker_manager
RUN apk add --no-cache musl-dev pkgconfig openssl-dev openssl-libs-static protoc protobuf-dev
RUN cargo build --release -p mc_server_seeker_manager


FROM alpine:latest AS mc_server_seeker_worker
WORKDIR /app
RUN apk add --no-cache masscan ca-certificates libgcc
COPY --from=builder-mc_server_seeker_worker /workspace/target/release/mc_server_seeker_worker ./mc_server_seeker_worker
CMD ["./mc_server_seeker_worker"]


FROM alpine:latest AS mc_server_seeker_manager
WORKDIR /app
RUN apk add --no-cache ca-certificates libgcc
COPY --from=builder-mc_server_seeker_manager /workspace/target/release/mc_server_seeker_manager ./mc_server_seeker_manager
CMD ["./mc_server_seeker_manager"]
