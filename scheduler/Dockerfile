
FROM rust:1.79 as builder

WORKDIR /app
COPY . .

ENV ACCOUNT_API_KEY "apikeytest"
ENV DATABASE_URL "postgresql://postgres:postgres@192.168.0.40:8000/nettuscheduler"
ENV PORT 3000

RUN cargo install sqlx-cli --locked

RUN sqlx migrate run --source /app/crates/infra/migrations

FROM debian:bullseye-slim

RUN apt-get update && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/cargo/bin/nettu-scheduler /usr/local/bin/nettu-scheduler

CMD ["nettu-scheduler"]