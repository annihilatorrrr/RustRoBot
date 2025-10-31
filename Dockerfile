FROM rust:1.91.0-alpine3.22 as builder
WORKDIR /RustRoBot
RUN apk update && apk upgrade --available && sync && apk add --no-cache --virtual .build-deps musl-dev libressl-dev build-base pkgconfig
RUN apk add --no-cache ca-certificates
COPY . .
RUN cargo build --release
FROM scratch
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=builder /RustRoBot/target/release/rustrobot /rustrobot
ENTRYPOINT ["/rustrobot"]
