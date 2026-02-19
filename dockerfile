# ---- Build Stage ----
FROM rust:1.75 as builder
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

# ---- Runtime Stage ----
FROM gcr.io/distroless/cc-debian12

WORKDIR /app
COPY --from=builder /app/target/release/tee_time_tracker_v2 /app/server

ENV PORT=8080

CMD ["/app/server"]
