Installation
============

The ``fastobo`` Python module is implemented in Rust, but the Rust compiler
is only required if your platform does not have precompiled wheels available.
Currently, we provide wheels for the following platforms:

* `manylinux1-x86_64` (Python 3.5, 3.6 and 3.7)
* `macosx-x86_64` (Python 3.6 and 3.7)

If your platform is not listed above, you will need to have the Rust compiler
installed. See `documentation on rust-lang.org <https://forge.rust-lang.org/other-installation-methods.html>`_
to learn how to install Rust on your machine.

Installation is supported through pip::

  $ pip install fastobo --user
