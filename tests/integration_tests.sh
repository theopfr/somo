#!/usr/bin/env bash
set -eu

docker build -t somo-it-tests ./tests/setup

docker run --rm \
    -v $(pwd)/Cargo.toml:/app/Cargo.toml \
    -v $(pwd)/Cargo.lock:/app/Cargo.lock \
    -v $(pwd)/src:/app/src \
    -v $(pwd)/tests:/app/tests \
    -v cargo-cache:/root/.cargo/registry \
    -v cargo-target:/app/target \
    somo-it-tests \
    bash -c "/usr/local/bin/mock_processes.sh & sleep 10 && cargo test --test integration_tests -- --ignored"
