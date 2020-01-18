#!/bin/sh -e

. $(dirname $(dirname $0))/functions.sh

# --- Using proper Python executable -----------------------------------------

if [ ! "$PYTHON" = "pypy3.7" ]; then
  eval "$(pyenv init -)"
  pyenv shell $(pyenv versions --bare)
fi

# --- Test -------------------------------------------------------------------

python setup.py test
