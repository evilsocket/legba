# build stage
FROM rust:bookworm as build

# create a new empty shell project
RUN USER=root cargo new --bin legba
WORKDIR /legba

# copy contents and cache dependencies
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release
RUN rm src/*.rs
COPY ./src ./src

# build
RUN rm ./target/release/deps/legba*
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/* 
RUN update-ca-certificates
COPY --from=build /legba/target/release/legba /usr/bin/legba
ENTRYPOINT ["/usr/bin/legba"]