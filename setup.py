#!/usr/bin/env python3

import configparser
import os

import setuptools
import setuptools_rust as rust
from setuptools.command.sdist import sdist as _sdist
from setuptools_rust.utils import get_rust_version


class sdist(_sdist):

    def run(self):
        # build `pyproject.toml` from `setup.cfg`
        c = configparser.ConfigParser()
        c.add_section("build-system")
        c.set("build-system", "requires", str(self.distribution.setup_requires))
        c.set("build-system", 'build-backend', '"setuptools.build_meta"')
        with open("pyproject.toml", "w") as pyproject:
            c.write(pyproject)

        # run the rest of the packaging
        _sdist.run(self)


if "nightly" in get_rust_version().prerelease:
    features = ["extension-module", "nightly"]
else:
    features = ["extension-module"]


setuptools.setup(
    setup_requires=["setuptools", "setuptools_rust"],
    cmdclass=dict(sdist=sdist),
    rust_extensions=[rust.RustExtension(
        "fastobo",
        path="Cargo.toml",
        binding=rust.Binding.PyO3,
        strip=rust.Strip.Debug,
        features=features,
    )],
)
