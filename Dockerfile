FROM rust:1.77.1-slim-bookworm as builder
WORKDIR /RustRoBot
RUN apt update && apt upgrade -y && apt install sudo -y && sudo apt install apt-utils build-essential libssl-dev libc-dev pkg-config -y
COPY . .
RUN cargo build --release
RUN echo ls
FROM alpine:3.19.1
RUN apk update && apk upgrade --available && sync && apk add --no-cache --virtual .build-deps openssl-dev musl-dev
COPY --from=builder /RustRoBot/target/release/rustrobot /RustRoBot
ENTRYPOINT ["/RustRoBot"]
