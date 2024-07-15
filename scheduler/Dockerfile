# FROM ubuntu:20.04

# ARG DEBIAN_FRONTEND=noninteractive

# # Update default packages
# RUN apt-get update \
#     && apt-get install -y build-essential curl pkg-config libssl-dev \
#     && apt-get -y dist-upgrade \
#     && rm -rf /var/lib/apt/lists/*

# ENV PATH="/root/.cargo/bin:${PATH}"

# WORKDIR /home/rust/

# COPY . .

# ENV DATABASE_URL "postgresql://postgres:postgres@172.17.0.1:5432/nettuscheduler"

# RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
#     && cargo build --release \
#     && cp /home/rust/target/release/nettu_scheduler . \
#     && cp /home/rust/target/release/migrate . \
#     && rm -rf /home/rust/target \
#     && rustup self uninstall -y

# ENTRYPOINT ["/bin/sh", "-c" , "./migrate && ./nettu_scheduler"]

# FROM alpine:latest 

# RUN apk add --no-cache cargo postgresql-dev

# ENV ACCOUNT_API_KEY "apikeytest"
# ENV DATABASE_URL "postgresql://postgres:postgres@192.168.0.40:8000/nettuscheduler"
# ENV PORT 3000

# WORKDIR /app

# COPY . .

# RUN cargo install sqlx-cli

# RUN ~/.cargo/bin/sqlx migrate run --source /app/crates/infra/migrations

# CMD ["cargo", "build", "--release"]

# EXPOSE 3000

# ENTRYPOINT ["/bin/sh", "-c" ,"/app/target/release/nettu_scheduler"]


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