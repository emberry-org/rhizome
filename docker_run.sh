#!/bin/sh
if ! [ -f "server.crt" ]; then
    echo "Missing \"server.crt\""
    exit
fi
if ! [ -f "server.key" ]; then
    echo "Missing \"server.key\""
    exit
fi

cargo build --release
docker build . -t rhizome
docker run --name rhizome --restart unless-stopped -dp 9999:9999 -p 10000:10000 rhizome
