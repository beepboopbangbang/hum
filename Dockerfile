FROM ekidd/rust-musl-builder AS builder

ADD --chown=rust:rust Cargo.toml Cargo.lock ./
ADD --chown=rust:rust src ./src
RUN cargo build --release

FROM alpine:latest

RUN apk add --no-cache ca-certificates
COPY --from=builder /home/rust/src/target/x86_64-unknown-linux-musl/release/hum /usr/local/bin/

ENV RUST_LOG="hum=info"
USER nobody
EXPOSE 3030
CMD ["/usr/local/bin/hum"]
