#!/usr/bin/env python3

import configparser
import os
import shutil
import subprocess
import sys
import urllib.request
from distutils.errors import DistutilsPlatformError
from distutils.log import INFO

import setuptools
import setuptools_rust as rust
from setuptools.command.sdist import sdist as _sdist
from setuptools_rust.build import build_rust as _build_rust
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


class build_rust(_build_rust):

    def run(self):

        try:
            rustc = get_rust_version()
            nightly = rustc.prerelease is not None and "nightly" in rustc.prerelease
        except DistutilsPlatformError:
            if sys.platform in ("linux", "darwin"):
                self.setup_temp_rustc_unix(toolchain="nightly", profile="minimal")
                nightly = True
            else:
                nightly = False

        if self.inplace:
            self.extensions[0].strip = rust.Strip.No
        if nightly:
            self.extensions[0].features.append("nightly")

        _build_rust.run(self)


    def setup_temp_rustc_unix(self, toolchain, profile):

        rustup_sh = os.path.join(self.build_temp, "rustup.sh")
        os.environ["CARGO_HOME"] = os.path.join(self.build_temp, "cargo")
        os.environ["RUSTUP_HOME"] = os.path.join(self.build_temp, "rustup")

        self.mkpath(os.environ["CARGO_HOME"])
        self.mkpath(os.environ["RUSTUP_HOME"])

        self.announce("downloading rustup.sh install script", level=INFO)
        with urllib.request.urlopen("https://sh.rustup.rs") as res:
            with open(rustup_sh, "wb") as dst:
                shutil.copyfileobj(res, dst)

        self.announce("installing Rust compiler to {}".format(self.build_temp), level=INFO)
        subprocess.call([
            "sh",
            rustup_sh,
            "-y",
            "--default-toolchain",
            toolchain,
            "--profile",
            profile,
            "--no-modify-path"
        ])

        self.announce("updating $PATH variable to use local Rust compiler", level=INFO)
        os.environ["PATH"] = ":".join([
            os.path.abspath(os.path.join(os.environ["CARGO_HOME"], "bin")),
            os.environ["PATH"]
        ])




setuptools.setup(
    setup_requires=["setuptools", "setuptools_rust"],
    cmdclass=dict(sdist=sdist, build_rust=build_rust),
    rust_extensions=[rust.RustExtension(
        "fastobo",
        path="Cargo.toml",
        binding=rust.Binding.PyO3,
        strip=rust.Strip.Debug,
        features=["extension-module"],
    )],
)
