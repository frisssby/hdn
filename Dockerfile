FROM rust:1.64

WORKDIR /hdn
COPY src src
COPY Cargo.toml Cargo.toml

RUN cargo install --path .

ENTRYPOINT ["hdn", "--config", "/tmp/config.json"]