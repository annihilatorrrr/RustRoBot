FROM rust:1.83.0-alpine3.20 as builder
WORKDIR /RustRoBot
RUN apk update && apk upgrade --available && sync && apk add --no-cache --virtual .build-deps musl-dev libressl-dev build-base pkgconfig
COPY . .
RUN cargo build --release
FROM alpine:3.21.0
RUN apk update && apk upgrade --available && sync
COPY --from=builder /RustRoBot/target/release/rustrobot /rustrobot
ENTRYPOINT ["/rustrobot"]
