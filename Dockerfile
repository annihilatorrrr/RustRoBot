FROM rust:1.77.1-slim-bookworm AS builder
WORKDIR /RustRoBot
RUN apt update && apt upgrade -y && apt install build-essential libssl-dev -y && rm -rf /var/lib/apt/lists/*
COPY . .
RUN cargo build --release
FROM alpine:3.19.1
RUN apk update && apk add --no-cache --virtual .build-deps openssl-dev musl-dev
COPY --from=builder /RustRoBot/target/release/rustrobot /rustrobot
ENTRYPOINT ["/rustrobot"]
