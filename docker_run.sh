#!/bin/sh
if ! [ -f "server.crt" ]; then
    echo "Missing \"server.crt\""
    exit
fi
if ! [ -f "server.key" ]; then
    echo "Missing \"server.key\""
    exit
fi

echo "making sure $HOME/.cargo/registry exists"
echo " --> mkdir $HOME/.cargo/registry -p"
mkdir $HOME/.cargo/registry -p
echo "run rust container to generate a Dockerfile compatible binary"
echo " --> docker run --rm --user \"$(id -u)\":\"$(id -g)\" -it -v \"$HOME/.cargo/registry\":/usr/local/cargo/registry \ -v \"$(pwd)\":/usr/src/myapp -w /usr/src/myapp rust cargo build --release"
docker run --rm --user "$(id -u)":"$(id -g)" -it -v "$HOME/.cargo/registry":/usr/local/cargo/registry \
-v "$(pwd)":/usr/src/myapp -w /usr/src/myapp rust cargo build --release

echo "build rhizome image"
echo " --> docker build . -t rhizome"
docker build . -t rhizome
echo "run rhizome with default ports, name=rhizome, autorestart"
echo " --> docker run --name rhizome --restart unless-stopped -dp 9999:9999 -p 10000:10000/udp rhizome"
docker run --name rhizome --restart unless-stopped -dp 9999:9999 -p 10000:10000/udp rhizome
