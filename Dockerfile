FROM rust:1.77.1-slim-bookworm as builder
WORKDIR /RustRoBot
RUN apt update && apt upgrade -y && apt install build-essential libssl-dev libc-dev pkg-config -y && rm -rf /var/lib/apt/lists/*
COPY . .
ENTRYPOINT cargo run --package rustrobot --bin rustrobot --color=never --release
