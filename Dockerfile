FROM rust:1.77.1-alpine3.19 as builder
WORKDIR /RustRoBot
RUN apk update && apk upgrade --available && sync && apk add --no-cache --virtual .build-deps pkgconfig openssl-dev
COPY . .
RUN cargo build --release
FROM alpine:3.19.1
RUN apk update && apk upgrade --available && sync
COPY --from=builder /RustRoBot/target/release/rustrobot /RustRoBot
ENTRYPOINT ["/RustRoBot"]
