#!/bin/sh -e

. $(dirname $(dirname $0))/functions.sh

# --- Using proper Python executable -----------------------------------------

if [ ! "$PYTHON" = "pypy3" ]; then
  eval "$(pyenv init -)"
  pyenv shell ${PYTHON#python}-dev
fi

# --- Test -------------------------------------------------------------------

python setup.py test
