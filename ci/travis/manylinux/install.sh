#!/bin/sh -e

. $(dirname $(dirname $0))/functions.sh


# --- Install Rust -----------------------------------------------------------

log Installing Rust nightly
docker exec -it manylinux sh -c \
  'curl -sSf https://build.travis-ci.org/files/rustup-init.sh | sh -s -- --default-toolchain=nightly -y'

# --- Install Python deployment dependencies ---------------------------------

log Installing Python requirements
pip install -r ci/requirements.txt
