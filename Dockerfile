FROM rust:1.77.1-slim-bookworm as builder
WORKDIR /RustRoBot
RUN apt update && apt upgrade -y && apt install sudo -y && sudo apt install apt-utils build-essential libssl-dev libc-dev pkg-config -y --no-install-recommends && rm -rf /var/lib/apt/lists/*
COPY . .
ENV RUSTFLAGS="-C target-cpu=native -C opt-level=3 -C link-arg=-s"
RUN cargo build --release
FROM alpine:3.19.1
RUN apk update && apk add --no-cache --virtual .build-deps openssl-dev musl-dev
COPY --from=builder /RustRoBot/target/release/rustrobot ./
ENTRYPOINT ["./rustrobot"]
