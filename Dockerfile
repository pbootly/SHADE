FROM rust:1-bullseye AS builder
RUN apt update && apt install -y libssl-dev pkg-config

WORKDIR /usr/src/shade

COPY . .

RUN cargo build --release

FROM debian:bullseye-slim
COPY --from=builder /usr/src/shade/target/release/shade /usr/local/bin/shade

ENTRYPOINT ["shade"]
CMD ["server"]

