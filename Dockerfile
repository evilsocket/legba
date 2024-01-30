FROM rust:bullseye as builder

RUN apt-get update && apt-get install -y libsmbclient-dev libssl-dev ca-certificates cmake git

WORKDIR /app
ADD . /app
RUN cargo build --release --features http_relative_paths

FROM debian:bullseye
RUN apt-get update && apt-get install -y libsmbclient libssl-dev ca-certificates
COPY --from=builder /app/target/release/legba /usr/bin/legba
ENTRYPOINT ["/usr/bin/legba"]