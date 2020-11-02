FROM rust:latest AS builder

WORKDIR /server
COPY . /server/
RUN rustup default nightly
RUN cargo update
RUN cargo build --release

FROM alpine:latest
# EXPOSE 80
COPY --from=builder /server/target/release/table_tennis /usr/local/bin/table_tennis
RUN table_tennis
