#!/bin/sh -e

. $(dirname $(dirname $0))/functions.sh

# --- Install Python ---------------------------------------------------------

if [ "$PYTHON" = "python3.7" ]; then
  log Updating Python to v${PYTHON#python}
  brew unlink python
  brew install https://raw.githubusercontent.com/Homebrew/homebrew-core/master/Formula/python.rb
else
  log Using Python ${PYTHON#python}
fi


# --- Install Rust -----------------------------------------------------------

log Installing Rust nightly
curl -sSf https://build.travis-ci.org/files/rustup-init.sh | sh -s -- --default-toolchain=nightly -y


# --- Install Python requirements --------------------------------------------

log Installing Python requirements
$PYTHON -m pip install -r "$TRAVIS_BUILD_DIR/ci/requirements.txt"


# --- Setup sccache ----------------------------------------------------------

LATEST=$(cargo search sccache | grep sccache | cut -f2 -d"\"")
LOCAL=$(sccache --version 2>/dev/null | cut -f2 -d" " || echo "none")

if [ "$LATEST" != "$LOCAL" ]; then
	log Downloading sccache v$LATEST
  URL="https://github.com/mozilla/sccache/releases/download/${LATEST}/sccache-${LATEST}-x86_64-apple-darwin.tar.gz"
	curl -SsL $URL | tar xzv -C /tmp
	mkdir -p "$HOME/.cargo/bin"
	mv "/tmp/sccache-${LATEST}-x86_64-apple-darwin/sccache" "$HOME/.cargo/bin/sccache"
	chmod +x "$HOME/.cargo/bin/sccache"
else
	log Using cached sccache v$LOCAL
fi
