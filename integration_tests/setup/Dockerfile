FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    netcat-openbsd \
    procps \
    curl \
    ca-certificates \
    pkg-config \
    libssl-dev \
    build-essential \
 && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

COPY mock_processes.sh /usr/local/bin/mock_processes.sh
RUN chmod +x /usr/local/bin/mock_processes.sh

WORKDIR /app
