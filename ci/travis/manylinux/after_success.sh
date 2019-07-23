#!/bin/sh -e

. $(dirname $(dirname $0))/functions.sh

# --- Patch version number ---------------------------------------------------

if [ -z "$TRAVIS_TAG" ]; then
  VERSION=$(python setup.py --version)-dev$(git rev-list --count --all)
  sed -i'.BAK' -e "s/version = $(python setup.py --version)/version = $VERSION/g" setup.cfg
fi

# --- Wheels -----------------------------------------------------------------

if [ ! -z "$TRAVIS_TAG" ]; then
  log Building wheel
  CP=cp$(echo $TRAVIS_PYTHON_VERSION | sed 's/\.//')
  docker exec -it manylinux sh /io/ci/travis/manylinux/_after_success.sh $CP
fi
