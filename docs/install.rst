Installation
============

The ``fastobo`` Python module is implemented in Rust, but the Rust compiler
is only required if your platform does not have precompiled wheels available.
Currently, we provide wheels for the following platforms:

* **Linux x86-64**: CPython 3.5, 3.6, 3.7, and PyPy 3.7
* **OSX x86-64**: CPython 3.6, 3.7, and PyPy 3.7
* **Windows x86-64**: CPython 3.5, 3.6, 3.7

If your platform is not listed above, you will need to have the Rust compiler
installed. See `documentation on rust-lang.org <https://forge.rust-lang.org/other-installation-methods.html>`_
to learn how to install Rust on your machine.

Installation is supported through pip::

  $ pip install fastobo --user

Note that this will install a static library that have been built with most
feature flags disabled for compatibility purposes. If you wish to build the
optimized library from source, with all feature flags enabled, make rust to
have Rust installed, and run::

  $ RUSTFLAGS="-Ctarget-cpu=native" pip install fastobo --user --no-binary :all:
