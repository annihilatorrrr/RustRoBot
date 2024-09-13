FROM rust:1.81.0-alpine3.20 as builder
WORKDIR /RustRoBot
RUN apk update && apk upgrade --available && sync && apk add --no-cache --virtual .build-deps musl-dev openssl-dev libressl-dev build-base pkgconfig
COPY . .
RUN cargo build --release
FROM alpine:3.20
RUN apk update && apk upgrade --available && sync
COPY --from=builder /RustRoBot/target/release/rustrobot /rustrobot
ENTRYPOINT ["/rustrobot"]
