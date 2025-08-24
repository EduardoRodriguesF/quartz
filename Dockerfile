FROM rust:1.89-slim

WORKDIR /app

RUN apt-get update && \
  apt-get install -y \
    libssl-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

COPY . .
