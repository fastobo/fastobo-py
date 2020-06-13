#!/bin/sh -e

. $(dirname $(dirname $0))/functions.sh


# --- Install Rust -----------------------------------------------------------

log Installing Rust nightly
curl -sSf https://build.travis-ci.org/files/rustup-init.sh | sh -s -- --default-toolchain=$RUST_TOOLCHAIN -y


# --- Installing cargo-tarpaulin ---------------------------------------------

log Downloading \`cargo-tarpaulin\` binary from GitHub
URL=https://github.com/xd009642/tarpaulin/releases/download/0.13.3/cargo-tarpaulin-0.13.3-travis.tar.gz
curl -SsL $URL | tar xvz -C $HOME/.cargo/bin
