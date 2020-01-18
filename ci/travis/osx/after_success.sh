#!/bin/sh -e

. $(dirname $(dirname $0))/functions.sh

# --- Using proper Python executable -----------------------------------------

if [ ! "$PYTHON" = "pypy3.7" ]; then
  eval "$(pyenv init -)"
  pyenv shell $(pyenv versions --bare)
fi

# --- Patch version number ---------------------------------------------------

if [ -z "$TRAVIS_TAG" ]; then
  VERSION=$($PYTHON setup.py --version)-dev$(git rev-list --count --all)
  sed -i'.BAK' -e "s/version = $($PYTHON setup.py --version)/version = $VERSION/g" setup.cfg
fi

# --- Wheels -----------------------------------------------------------------

if [ ! -z "$TRAVIS_TAG" ]; then
  log Building wheel
  python setup.py sdist bdist_wheel
fi
