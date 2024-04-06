FROM rust:latest AS builder
WORKDIR /usr/src/app
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src
COPY . .
RUN cargo build --release
FROM alpine:latest
COPY --from=builder /usr/src/app/target/release/RustRoBot /usr/local/bin/RustRoBot
CMD ["/usr/local/bin/RustRoBot"]
