FROM rust:latest AS builder
COPY . .
RUN cargo build --release

FROM debian:buster-slim
RUN apt-get update
RUN apt-get install -y libssl-dev
COPY --from=builder /target/release/get_apply_set /usr/local/bin/get_apply_set

ENTRYPOINT [ "/usr/local/bin/get_apply_set" ]
