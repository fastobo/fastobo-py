#!/usr/bin/env python3

import os

import setuptools
import setuptools_rust as rust

setuptools.setup(
    setup_requires=["setuptools", "setuptools_rust"],
    rust_extensions=[rust.RustExtension(
        "fastobo",
        path="Cargo.toml",
        binding=rust.Binding.PyO3,
        strip=rust.Strip.Debug
    )],
)
