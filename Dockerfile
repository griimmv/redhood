FROM rust:1.85-slim-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --locked

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/* && \
    addgroup --system redhood && adduser --system --ingroup redhood redhood
COPY --from=builder --chown=redhood:redhood /app/target/release/redhood /usr/local/bin/redhood
USER redhood
EXPOSE 8080
ENTRYPOINT ["redhood"]
