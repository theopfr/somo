#!/usr/bin/env bash

# This script generates a mock procfs directory by spinning up a Docker container which spawns some network bound processes
# using the 'tests/setup/init_processes.sh' script and copies the relevant procfs files to the host mock directory.

set -eu pipefail

CONTAINER_NAME="mock-procfs-generator"
MOCK_DIR="tests/mock/"

rm -rf "$MOCK_DIR/proc/"
mkdir -p "$MOCK_DIR/proc/net/"

echo "Building the Docker image..."
docker build -t mock-procfs-image tests/setup/

echo "Running the container..."
docker run --name "$CONTAINER_NAME" --rm -d mock-procfs-image

echo "Waiting for the container to start and processes to bind..."
sleep 5

echo "Identifying process PIDs inside the container..."
PIDS=$(docker exec "$CONTAINER_NAME" ps -o pid,comm | grep -E 'nc$' | awk '{print $1}')

if [ -z "$PIDS" ]; then
    echo "No 'nc' processes found. The container may not have started correctly."
    docker stop "$CONTAINER_NAME"
    exit 1
fi

echo "Found PIDs: $PIDS"

# Copy /proc/net files
for netfile in tcp tcp6 udp udp6; do
    echo "Copying /proc/net/$netfile"
    docker exec "$CONTAINER_NAME" cat "/proc/net/$netfile" > "$MOCK_DIR/proc/net/$netfile"
done

# Copy per-PID files (cmdline, stat, exe, fd)
for pid in $PIDS; do
    pid_dir="$MOCK_DIR/proc/$pid"
    mkdir -p "$pid_dir/fd"

    echo "Copying /proc/$pid/cmdline"
    docker exec "$CONTAINER_NAME" cat "/proc/$pid/cmdline" > "$pid_dir/cmdline" || true

    echo "Copying /proc/$pid/stat"
    docker exec "$CONTAINER_NAME" cat "/proc/$pid/stat" > "$pid_dir/stat" || true

    echo "Copying symlink /proc/$pid/exe"
    exe_target=$(docker exec "$CONTAINER_NAME" readlink "/proc/$pid/exe" || true)
    if [ -n "$exe_target" ]; then
        ln -sf "$exe_target" "$pid_dir/exe"
    fi

    echo "Copying /proc/$pid/fd symlinks"
    fd_list=$(docker exec "$CONTAINER_NAME" ls -1 "/proc/$pid/fd" 2>/dev/null || true)

    for fd in $fd_list; do
        target=$(docker exec "$CONTAINER_NAME" readlink "/proc/$pid/fd/$fd" || true)
        if [ -n "$target" ]; then
            ln -sf "$target" "$pid_dir/fd/$fd"
        fi
    done

done

echo "Files copied successfully to '$MOCK_DIR'."

echo "Stopping and removing the container..."
docker stop "$CONTAINER_NAME"