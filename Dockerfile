FROM rust:1-buster
COPY riskytrees /app/riskytrees
COPY cas/rds-combined-ca-bundle.pem /app/riskytrees/rds-combined-ca-bundle.pem

WORKDIR /app/riskytrees

RUN cargo build
ENTRYPOINT ["cargo"]
CMD ["run"]
