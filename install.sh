#!/usr/bin/env bash

cargo build --release
sudo install -Dm755 ./target/release/tohorank /usr/bin/tohorank
install -Dm644 ./src/touhous.txt $HOME/.tohorank/touhous.txt