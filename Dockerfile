FROM rust:1.76 as builder
WORKDIR /Backend
COPY . .

RUN cargo build --release

EXPOSE 8282

CMD ["./target/release/backend"]