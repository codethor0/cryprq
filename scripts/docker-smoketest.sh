#!/bin/bash
# Automated two-node Docker test for cryprq
set -e

IMAGE=cryprq-test
BINARY=target/x86_64-unknown-linux-musl/release/cryprq

# Build Docker image
if [ ! -f "$BINARY" ]; then
  echo "Building static binary..."
  docker build -f Dockerfile -t $IMAGE .
else
  echo "Using existing binary for Docker image."
  docker build -f Dockerfile -t $IMAGE .
fi

# Start listener node
LISTEN_ADDR="/ip4/0.0.0.0/udp/9999/quic-v1"
LISTEN_CONTAINER=cryprq-listen

docker run -d --name $LISTEN_CONTAINER $IMAGE --listen $LISTEN_ADDR
sleep 2

# Get listener container IP
LISTEN_IP=$(docker inspect -f '{{range.NetworkSettings.Networks}}{{.IPAddress}}{{end}}' $LISTEN_CONTAINER)
PEER_ADDR="/ip4/$LISTEN_IP/udp/9999/quic-v1"

echo "Listener running at $PEER_ADDR"

docker run --rm $IMAGE --peer $PEER_ADDR

docker logs $LISTEN_CONTAINER

docker rm -f $LISTEN_CONTAINER
