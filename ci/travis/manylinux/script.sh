#!/bin/sh -e

. $(dirname $(dirname $0))/functions.sh

# --- Test -------------------------------------------------------------------

if [ ! "$TRAVIS_PYTHON_VERSION" = "pypy3" ]; then
  TAG=cp$(echo $TRAVIS_PYTHON_VERSION | sed 's/\.//')
fi

log Running test with $TAG
docker exec -it manylinux sh /io/ci/travis/manylinux/_script.sh $TAG
