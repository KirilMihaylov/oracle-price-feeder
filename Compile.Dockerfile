FROM rust:1.65.0-alpine

VOLUME ["/artifacts", "/code", "/code/target", "/usr/local/cargo"]

RUN ["apk", "add", "musl-dev"]

WORKDIR "/code"

ENTRYPOINT ["sh", "-c", "cargo build --release -p market-data-feeder --target x86_64-unknown-linux-musl && cargo build --release -p alarms-dispatcher --target x86_64-unknown-linux-musl && cp /code/target/x86_64-unknown-linux-musl/release/feeder /artifacts && cp /code/target/x86_64-unknown-linux-musl/release/alarms-dispatcher /artifacts"]