#!/usr/bin/env bash

set -eu pipefail

CONTAINER_NAME="mock_network_processes"

echo "Building the Docker image..."
docker build -t mock-procfs-image mock_procfs

echo "Running the container..."
docker run --name "$CONTAINER_NAME" --rm -d mock-procfs-image

echo "Waiting for the container to start and processes to bind..."
sleep 5

echo "Copying /proc/net files from the container to the host..."
docker exec "$CONTAINER_NAME" cat /proc/net/tcp > mock_procfs/net/tcp
docker exec "$CONTAINER_NAME" cat /proc/net/tcp6 > mock_procfs/net/tcp6
docker exec "$CONTAINER_NAME" cat /proc/net/udp > mock_procfs/net/udp
docker exec "$CONTAINER_NAME" cat /proc/net/udp6 > mock_procfs/net/udp6

echo "Files copied successfully."

echo "Stopping the container..."
docker stop "$CONTAINER_NAME"
