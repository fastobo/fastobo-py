#!/bin/sh -e

. $(dirname $(dirname $0))/functions.sh

# --- Patch version number ---------------------------------------------------

if [ -z "$TRAVIS_TAG" ]; then
  VERSION=$($PYTHON setup.py --version)-dev$(git rev-list --count --all)
  sed -i'.BAK' -e "s/version = $($PYTHON setup.py --version)/version = $VERSION/g" setup.cfg
fi

# --- Wheels -----------------------------------------------------------------

log Building wheel
$PYTHON setup.py sdist bdist_wheel

# --- Deploy to PyPI ---------------------------------------------------------

log Deploying wheel to PyPI
twine upload --skip-existing dist/*.whl dist/*.tar.gz ;;
