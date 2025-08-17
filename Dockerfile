FROM rust:bullseye AS builder

RUN apt-get update && apt-get install -y cmake git

WORKDIR /app
ADD . /app
RUN cargo build --release

FROM debian:bullseye
COPY --from=builder /app/target/release/legba /usr/bin/legba
ENTRYPOINT ["/usr/bin/legba"]