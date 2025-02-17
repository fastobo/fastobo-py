Installation
============

.. highlight:: console

Precompiled Wheels
------------------

The ``fastobo`` Python module is implemented in Rust, but the Rust compiler
is only required if your platform does not have precompiled wheels available.
Currently, we provide `wheels <https://pythonwheels.com/>`_ for the following
platforms:

* **Linux**: *x86-64* and *Aarch64*
* **MacOS**: *x86-64* and *Aarch64*
* **Windows**: *x86-64* only.

The supported Python versions are provided with the 
`cibuildwheel <https://cibuildwheel.pypa.io>`_ tool. Downloading and
installing from a wheel is then as simple as::

  $ pip install fastobo --user

If your platform and implementation is not listed above, you will need to build
from source (see next section). 


Conda package
-------------

``fastobo`` is also available for `Conda <https://anaconda.org>`_ in the
``conda-forge`` channel::

  $ conda install conda-forge::fastobo 


Piwheels
^^^^^^^^

``fastobo`` works on Raspberry Pi computers, and pre-built wheels are compiled 
for `armv7l` on `piwheels <https://www.piwheels.org/project/fastobo/>`_.
Run the following command to install these instead of compiling from source:

.. code:: console

   $ pip3 install fastobo --extra-index-url https://www.piwheels.org/simple

Check the `piwheels documentation <https://www.piwheels.org/faq.html>`_ for 
more information.


Building from source
--------------------

In order to build the code from source, you will need to have
the Rust compiler installed and available in your ``$PATH``. See
`documentation on rust-lang.org <https://forge.rust-lang.org/other-installation-methods.html>`_
to learn how to install Rust on your machine.

Then installing with ``pip`` will build the pacakge::

  $ pip install fastobo --user -v --no-binary :all:

**Be patient, it can take a long time on lower-end machine!**

Note that this will install a static library that have been built with most
feature flags disabled for compatibility purposes. If you wish to build the
optimized library from source, with all feature flags enabled, make sure to
have ``-C target-cpu=native`` in your ``$RUSTFLAGS`` environment while building::

  $ RUSTFLAGS="-Ctarget-cpu=native" pip install fastobo --user --no-binary :all:
