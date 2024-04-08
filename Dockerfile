FROM rust:latest AS builder
WORKDIR /RustRoBot
RUN apk update && apk upgrade --available && sync && apk add --no-cache --virtual .build-deps
COPY . .
RUN cargo build --release
FROM alpine:latest
RUN apk update && apk upgrade --available && sync
COPY --from=builder /RustRoBot/target/release/rustrobot /RustRoBot
ENTRYPOINT ["/RustRoBot"]
