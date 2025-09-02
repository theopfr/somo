#!/usr/bin/env bash

start_process() {
  local proto=$1
  local addr=$2
  local port=$3

  echo "Starting $proto bound process on [$addr]:$port"
  if [[ "$proto" == "tcp" ]]; then
    while true; do
      nc -l "$addr" "$port" >/dev/null 2>&1
    done &
  elif [[ "$proto" == "udp" ]]; then
    while true; do
      nc -lu "$addr" "$port" >/dev/null 2>&1
    done &
  else
    echo "Unknown protocol: $proto"
    exit 1
  fi
}

# IPv4 TCP
start_process tcp 0.0.0.0 5001
start_process tcp 0.0.0.0 5002

# IPv6 TCP
start_process tcp :: 5003
start_process tcp :: 5004

# IPv4 UDP
start_process udp 0.0.0.0 5005
start_process udp 0.0.0.0 5006

# IPv6 UDP
start_process udp :: 5007
start_process udp :: 5008

echo "All processes started. Container will now remain running."

wait
