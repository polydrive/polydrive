FROM rust:1.59 as builder

ENV PROTO_PATH=./proto

RUN USER=root cargo new --bin polydrive && rustup component add rustfmt

WORKDIR ./polydrive

COPY ./client/Cargo.toml ./Cargo.toml
COPY ./client/build.rs ./
COPY ./server/src/main/protobuf ./$PROTO_PATH

RUN cargo build --release && rm src/*.rs

COPY ./client/src ./src

RUN rm ./target/release/deps/polydrive* && \
    cargo build --release

FROM debian:buster-slim

WORKDIR /app

RUN apt-get update && \
    apt-get install -y ca-certificates && \
    groupadd polydrive && \
    useradd -g polydrive polydrive && \
    mkdir -p /home/polydrive && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /polydrive/target/release/polydrive ./

RUN chown -R polydrive:polydrive ./polydrive && chown -R polydrive:polydrive /home/polydrive

USER polydrive

ENTRYPOINT ["./polydrive", "--daemon", "--watch", "/home/polydrive"]