Installation
============

Precompiled Wheels
------------------

The ``fastobo`` Python module is implemented in Rust, but the Rust compiler
is only required if your platform does not have precompiled wheels available.
Currently, we provide `wheels <https://pythonwheels.com/>`_ for the following
platforms and implementations:

* **Linux x86-64**: CPython 3.5, 3.6, 3.7, 3.8, and PyPy3 7.1.0, 7.2.0, 7.3.0
* **OSX x86-64**: CPython 3.5, 3.6, 3.7, 3.8, and PyPy3 7.3.0
* **Windows x86-64**: CPython 3.5, 3.6, 3.7, 3.8

If your platform and implementation is not listed above, you will need to build
from source (see next section). Feel free to
`open an issue <https://github.com/fastobo/fastobo-py/issues>`_ as well!

Downloading and installing from a wheel is then as simple as:

  $ pip install fastobo --user


Bioconda package
----------------

``fastobo`` is also available in the
`Bioconda <https://anaconda.org/bioconda/fastobo>`_ channel of the ``conda``
package manager::

  $ conda install -c bioconda fastobo

Note that only Linux x86-64 is supported.


Building from source
--------------------

In order to build the code from source, you will need to have
the Rust compiler installed and available in your ``$PATH``. See
`documentation on rust-lang.org <https://forge.rust-lang.org/other-installation-methods.html>`_
to learn how to install Rust on your machine.

Then installing with ``pip`` will build the pacakge::

  $ pip install fastobo --user

 **Be patient, it can take a long time on lower-end machine!**

Note that this will install a static library that have been built with most
feature flags disabled for compatibility purposes. If you wish to build the
optimized library from source, with all feature flags enabled, make sure to
have ``-C target-cpu=native`` in your ``$RUSTFLAGS`` environment while building::

  $ RUSTFLAGS="-Ctarget-cpu=native" pip install fastobo --user --no-binary :all:
