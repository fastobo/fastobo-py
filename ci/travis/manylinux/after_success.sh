#!/bin/sh -e

. $(dirname $(dirname $0))/functions.sh

# --- Wheels -----------------------------------------------------------------

if [ ! -z "$TRAVIS_TAG" ]; then

  if [ "$TRAVIS_PYTHON_VERSION" = "pypy3" ]; then
    TAG="$TRAVIS_PYTHON_VERSION*-$TRAVIS_PYPY_VERSION"
    PYTHON_PREFIX="/opt/pypy/$TAG"
  else
    TAG=cp$(echo $TRAVIS_PYTHON_VERSION | sed 's/\.//')
    PYTHON_PREFIX="/opt/python/$TAG-*"
  fi

  log Building wheel with $TRAVIS_PYTHON_VERSION
  docker exec -it manylinux sh /io/ci/travis/manylinux/_after_success.sh "$PYTHON_PREFIX"
fi
