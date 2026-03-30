FROM rust:1.90-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --locked -p aoxcmd --bin aoxc --release

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --create-home --home-dir /var/lib/aoxchain --shell /usr/sbin/nologin aoxchain

WORKDIR /opt/aoxchain
COPY --from=builder /app/target/release/aoxc /usr/local/bin/aoxc

USER aoxchain
ENV AOXC_HOME=/var/lib/aoxchain

EXPOSE 26656 8545
ENTRYPOINT ["aoxc"]
CMD ["node-bootstrap"]
