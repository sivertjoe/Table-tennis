FROM rust:latest AS builder

RUN apt update && \
    apt dist-upgrade -y && \
    apt install -y musl-tools && \
    rustup target add x86_64-unknown-linux-musl

WORKDIR /server
COPY . /server/
RUN cargo install --target x86_64-unknown-linux-musl --path .

FROM alpine:latest
RUN apk add --no-cache pkgconfig openssl-dev ca-certificates linux-headers && update-ca-certificates
RUN apk add openssl

COPY --from=builder /usr/local/cargo/bin/table_tennis .
CMD ["./table_tennis"]
