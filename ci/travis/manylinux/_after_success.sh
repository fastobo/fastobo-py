#!/bin/sh -e

export PATH="$HOME/.cargo/bin:$PATH"
export PYBIN="$(echo ${1}/bin)"
export PYTHON_SYS_EXECUTABLE="$PYBIN/python"
export PYTHON_LIB=$(${PYBIN}/python -c "import sysconfig; print(sysconfig.get_config_var('LIBDIR'))")
export LIBRARY_PATH="$LIBRARY_PATH:$PYTHON_LIB"
export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:$PYTHON_LIB"

# compile wheels
cd /io
$PYTHON_SYS_EXECUTABLE setup.py sdist bdist_wheel

# move wheels to tempdir
mkdir -p /tmp/wheels
mkdir -p /tmp/repaired
mv /io/dist/*.whl -t /tmp/wheels

# Bundle external shared libraries into the wheels
for whl in /tmp/wheels/*.whl; do
  auditwheel repair "$whl" -w /tmp/repaired
done

# Fix potentially invalid tags in wheel name
for whl in /tmp/repaired/*.whl; do
  auditwheel addtag "$whl" -w /io/dist
done
