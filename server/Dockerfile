FROM rust:latest AS builder
ARG build_mode

RUN apt update && \
    apt dist-upgrade -y && \
    apt install -y musl-tools && \
    rustup target add x86_64-unknown-linux-musl

WORKDIR /server
COPY . /server/
RUN cargo install $build_mode --target x86_64-unknown-linux-musl --path .

FROM alpine:latest
RUN apk add --no-cache pkgconfig openssl-dev

COPY --from=builder /usr/local/cargo/bin/table_tennis .
CMD ["./table_tennis"]
