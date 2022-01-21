FROM rust:1-buster
COPY riskytrees /app/riskytrees

WORKDIR /app/riskytrees

RUN cargo build
ENTRYPOINT ["cargo", "run"]
