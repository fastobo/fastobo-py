#!/bin/sh -e

. $(dirname $(dirname $0))/functions.sh

# --- Install Python ---------------------------------------------------------

log Installing pyenv from GitHub
git clone https://github.com/pyenv/pyenv.git $PYENV_ROOT
eval "$(pyenv init -)"

if [ "$PYTHON" = "pypy3" ]; then
  log Installing PyPy3
  brew unlink python
  brew install pypy3
  ln -s /usr/local/bin/pypy3 /usr/local/bin/python3
else
  log Install Python v${PYTHON#python}
  pyenv install ${PYTHON#python}-dev
  pyenv shell ${PYTHON#python}-dev
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
