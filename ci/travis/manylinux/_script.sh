#!/bin/sh

set -e

export PYBIN="$(echo ${1}/bin)"
export PYTHON_SYS_EXECUTABLE="$PYBIN/python"
export PATH="$HOME/.cargo/bin:$PYBIN:$PATH"
export PYTHON_LIB=$(${PYBIN}/python -c "import sysconfig; print(sysconfig.get_config_var('LIBDIR'))")
export LIBRARY_PATH="$LIBRARY_PATH:$PYTHON_LIB"
export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:$PYTHON_LIB"

pip install -U setuptools setuptools-rust requests

cd /io
python setup.py build_ext --inplace -vv
python -m unittest discover -vv
