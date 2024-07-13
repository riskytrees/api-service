FROM rust:1-slim-buster
COPY riskytrees /app/riskytrees
COPY cas/global-bundle.pem /app/riskytrees/global-bundle.pem

WORKDIR /app/riskytrees

RUN cargo build --release
ENTRYPOINT ["cargo"]
CMD ["run", "--release"]
