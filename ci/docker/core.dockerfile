ARG OS_CODENAME=bookworm
# Build stage
FROM rust:slim-${OS_CODENAME} AS builder
COPY . /app
WORKDIR /app
RUN cargo build --release

# Final stage
FROM debian:${OS_CODENAME}-slim
COPY --from=builder /app/target/release/agimo /usr/local/bin/agimo
ENTRYPOINT ["agimo"]