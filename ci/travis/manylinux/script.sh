#!/bin/sh -e

. $(dirname $(dirname $0))/functions.sh

# --- Test -------------------------------------------------------------------

log Running test
CP=cp$(echo $TRAVIS_PYTHON_VERSION | sed 's/\.//')
docker exec -it manylinux sh /io/ci/travis/manylinux/_script.sh $CP
