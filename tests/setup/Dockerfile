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

COPY init_processes.sh /usr/local/bin/init_processes.sh
RUN chmod +x /usr/local/bin/init_processes.sh

ENTRYPOINT ["/usr/local/bin/init_processes.sh"]