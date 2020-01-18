#!/bin/sh -e

. $(dirname $(dirname $0))/functions.sh

# --- Using proper Python executable -----------------------------------------

if [ ! "$PYTHON" = "pypy3.7" ]; then
  log Activating pyenv
  eval "$(pyenv init -)"
  pyenv shell $(pyenv versions --bare)
fi

# --- Test -------------------------------------------------------------------

log Using $(python --version | head -n1 | cut -d' ' -f1,2)
python setup.py test
