FROM rust:latest AS builder
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release
FROM alpine:latest
COPY --from=builder /usr/src/app/target/release/rustrobot /usr/local/bin/rustrobot
CMD ["/usr/local/bin/rustrobot"]
