# ---------- Build Stage ----------
FROM rust:1.85 AS builder
WORKDIR /app

COPY . .

RUN cargo build --release

# ---------- Runtime Stage ----------
FROM gcr.io/distroless/cc-debian12
WORKDIR /app

COPY --from=builder /app/target/release/tee_time_tracker_v2 /app/server

# Copy resources if needed
COPY --from=builder /app/src/resources /app/src/resources

ENV PORT=8080
EXPOSE 8080

CMD ["/app/server"]