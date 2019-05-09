#!/bin/sh -e

. $(dirname $(dirname $0))/functions.sh

# --- Test -------------------------------------------------------------------

$PYTHON setup.py test
