# riskytrees API Service

This is the primary backend web service for managing complex, configurable, attack trees.

## Running Locally

### Pre-requisites

#### Required environment variables
You will first want to populate a `.env` file (or use `tests/mocklab_odic.env`) to ensure logins
and the primary session token signing key are set.

1. RISKY_TREES_GOOGLE_REDIRECT_URL
1. RISKY_TREES_GOOGLE_CLIENT_ID
1. RISKY_TREES_GOOGLE_CLIENT_SECRET
1. RISKY_TREES_GOOGLE_AUTH_URL
1. RISKY_TREES_GOOGLE_TOKEN_URL
1. RISKY_TREES_GOOGLE_ISSUER_URL
1. RISKY_TREES_GOOGLE_JWKS_URL

1. RISKY_TREES_GITHUB_REDIRECT_URL
1. RISKY_TREES_GITHUB_CLIENT_ID
1. RISKY_TREES_GITHUB_CLIENT_SECRET
1. RISKY_TREES_GITHUB_AUTH_URL
1. RISKY_TREES_GITHUB_TOKEN_URL
1. RISKY_TREES_GITHUB_ISSUER_URL
1. RISKY_TREES_GITHUB_JWKS_URL

1. RISKY_TREES_JWT_SECRET

#### Tool chain
1. You will need to install rust: https://www.rust-lang.org/tools/install
1. You will also need docker installed: https://www.docker.com/get-started/

### Building & Running
1. To setup the database, run `./tests/simulatedb.sh`
1. To build, `cd` into the riskytrees subdirectory and run: `cargo build`.
1. To run, execute: `export ROCKET_ADDRESS=0.0.0.0 && source ../tests/mocklab_odic.env && cargo run`. *(Replace `../tests/mocklab_odic.env` with the path to your .env file if you changed anything.)*

