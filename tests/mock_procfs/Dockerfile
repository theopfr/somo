FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    netcat-openbsd \
    procps \
 && rm -rf /var/lib/apt/lists/*

COPY mock_processes.sh /usr/local/bin/mock_processes.sh
RUN chmod +x /usr/local/bin/mock_processes.sh

ENTRYPOINT ["/usr/local/bin/mock_processes.sh"]
