#!/bin/sh -e

. $(dirname $(dirname $0))/functions.sh

# --- Launch manylinux container ---------------------------------------------

IMG="quay.io/pypa/manylinux1_x86_64"
log Launching 'manylinux' docker container
docker run -d -e TERM=$TERM -v $TRAVIS_BUILD_DIR:/io --name manylinux --rm -it $IMG sh
