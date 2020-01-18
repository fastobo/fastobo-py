#!/bin/sh -e

. $(dirname $(dirname $0))/functions.sh

# --- Install Python ---------------------------------------------------------

log Installing pyenv from GitHub
git clone https://github.com/pyenv/pyenv.git $PYENV_ROOT
eval "$(pyenv init -)"

if [ "$PYTHON" = "pypy3.7" ]; then
  log Installing PyPy v3.7
  brew unlink python
  brew install pypy3
  ln -s /usr/local/bin/pypy3 /usr/local/bin/python
elif [ "$PYTHON" = "pypy3.6" ]; then
  log Installing PyPy v3.6
  pyenv install pypy3.6-7.3.0
  pyenv shell pypy3.6-7.3.0
elif [ "$PYTHON" = "pypy3.5" ]; then
  log Installing PyPy v3.5
  pyenv install pypy3.5-7.0.0
  pyenv shell pypy3.5-7.0.0
else
  log Install Python v${PYTHON#python}
  pyenv install ${PYTHON#python}-dev
  pyenv shell ${PYTHON#python}-dev
fi

# --- Check system Python version --------------------------------------------

log Checking global Python version
python --version


# --- Install Rust -----------------------------------------------------------

log Installing Rust nightly
curl -sSf https://build.travis-ci.org/files/rustup-init.sh | sh -s -- --default-toolchain=nightly -y


# --- Install Python requirements --------------------------------------------

log Installing Python requirements
python -m pip install -r "$TRAVIS_BUILD_DIR/ci/requirements.txt"


# --- Setup cargo-cache ------------------------------------------------------

LATEST=$(cargo search cargo-cache | head -n1 | cut -f2 -d"\"")
LOCAL=$(cargo cache --version 2>/dev/null | cut -d" " -f2 || echo "none")

if [ "$LATEST" != "$LOCAL" ]; then
        log Installing cargo-cache v$LATEST
        cargo install -f cargo-cache --root "$HOME/.cargo"
else
        log Using cached cargo-cache v$LOCAL
fi
