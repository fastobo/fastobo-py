#!/bin/sh -e

. $(dirname $(dirname $0))/functions.sh

# --- Install Python ---------------------------------------------------------

if [ "$PYTHON" = "python3.7" ]; then
  log Updating Python to v${PYTHON#python}
  brew unlink python
  brew install https://raw.githubusercontent.com/Homebrew/homebrew-core/master/Formula/python.rb
elif [ "$PYTHON" = "pypy3" ]; then
  log Installing PyPy3
  brew unlink python
  brew install pypy3
  ln -s /usr/local/bin/pypy3 /usr/local/bin/python3
else
  log Using Python v${PYTHON#python}
fi


# --- Install Rust -----------------------------------------------------------

log Installing Rust nightly
curl -sSf https://build.travis-ci.org/files/rustup-init.sh | sh -s -- --default-toolchain=nightly -y


# --- Install Python requirements --------------------------------------------

log Installing Python requirements
$PYTHON -m pip install -r "$TRAVIS_BUILD_DIR/ci/requirements.txt"


# --- Setup cargo-cache ------------------------------------------------------

LATEST=$(cargo search cargo-cache | head -n1 | cut -f2 -d"\"")
LOCAL=$(cargo cache --version 2>/dev/null | cut -d" " -f2 || echo "none")

if [ "$LATEST" != "$LOCAL" ]; then
        log Installing cargo-cache v$LATEST
        cargo install -f cargo-cache --root "$HOME/.cargo"
else
        log Using cached cargo-cache v$LOCAL
fi

