FROM rust:1.97-slim-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --locked

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/* && \
    addgroup --system redhood && adduser --system --ingroup redhood redhood
WORKDIR /app
RUN mkdir -p /app/data && chown -R redhood:redhood /app
COPY --from=builder --chown=redhood:redhood /app/target/release/redhood /usr/local/bin/redhood
USER redhood
EXPOSE 8080
ENTRYPOINT ["redhood"]
