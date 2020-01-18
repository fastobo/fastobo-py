#!/bin/sh -e

. $(dirname $(dirname $0))/functions.sh

# --- Patch version number ---------------------------------------------------

if [ -z "$TRAVIS_TAG" ]; then
  VERSION=$(python setup.py --version)-dev$(git rev-list --count --all)
  sed -i'.BAK' -e "s/version = $(python setup.py --version)/version = $VERSION/g" setup.cfg
fi

# --- Wheels -----------------------------------------------------------------

if [ ! -z "$TRAVIS_TAG" ]; then

  if [ ! "$TRAVIS_PYTHON_VERSION" = "pypy3" ]; then
    TAG=cp$(echo $TRAVIS_PYTHON_VERSION | sed 's/\.//')
  fi

  log Building wheel with $TAG
  docker exec -it manylinux sh /io/ci/travis/manylinux/_after_success.sh $TAG
fi
