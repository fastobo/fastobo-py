#!/bin/sh -e

export PATH="$HOME/.cargo/bin:$PATH"
export PYTHON_SYS_EXECUTABLE="python${TRAVIS_PYTHON_VERSION}"
export PYTHON_LIB=$(${PYTHON_SYS_EXECUTABLE} -c "import sysconfig; print(sysconfig.get_config_var('LIBDIR'))")
export LIBRARY_PATH="$LIBRARY_PATH:$PYTHON_LIB"
export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:$PYTHON_LIB"

cargo tarpaulin -v --out Xml --ciserver travis-ci
