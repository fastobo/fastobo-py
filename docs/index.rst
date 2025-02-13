|Logo| ``fastobo`` |Stars|
==========================

.. |Logo| image:: /_images/logo.png
   :scale: 10%
   :class: dark-light

.. |Stars| image:: https://img.shields.io/github/stars/fastobo/fastobo-py.svg?style=social&maxAge=3600&label=Star
   :target: https://github.com/fastobo/fastobo-py/stargazers

*Faultless AST for Open Biomedical Ontologies in Python.*

|Actions| |License| |Source| |PyPI| |Wheel| |Bioconda| |Versions| |Implementation| |Changelog| |Docs| |Issues| |DOI| |Downloads|

.. |PyPI| image:: https://img.shields.io/pypi/v/fastobo.svg?style=flat-square&maxAge=300
   :target: https://pypi.org/project/fastobo

.. |Bioconda| image:: https://img.shields.io/conda/vn/bioconda/fastobo?style=flat-square&maxAge=3600
   :target: https://anaconda.org/bioconda/fastobo

.. |Actions| image:: https://img.shields.io/github/actions/workflow/status/fastobo/fastobo-py/test.yml?branch=master&style=flat-square&maxAge=600
   :target: https://github.com/fastobo/fastobo-py/actions

.. |Wheel| image:: https://img.shields.io/pypi/wheel/fastobo.svg?style=flat-square&maxAge=2678400
   :target: https://pypi.org/project/fastobo

.. |Versions| image:: https://img.shields.io/pypi/pyversions/fastobo.svg?style=flat-square&maxAge=300
   :target: https://travis-ci.org/fastobo/fastobo-py

.. |Changelog| image:: https://img.shields.io/badge/keep%20a-changelog-8A0707.svg?maxAge=2678400&style=flat-square
   :target: https://github.com/fastobo/fastobo-py/blob/master/CHANGELOG.md

.. |License| image:: https://img.shields.io/pypi/l/fastobo.svg?style=flat-square&maxAge=300
   :target: https://choosealicense.com/licenses/mit/

.. |Source| image:: https://img.shields.io/badge/source-GitHub-303030.svg?maxAge=3600&style=flat-square
   :target: https://github.com/fastobo/fastobo-py

.. |Implementation| image:: https://img.shields.io/pypi/implementation/fastobo.svg?style=flat-square&maxAge=600
   :target: https://pypi.org/project/fastobo/#files

.. |Docs| image:: https://img.shields.io/readthedocs/fastobo.svg?maxAge=3600&style=flat-square
   :target: https://fastobo.readthedocs.io/

.. |Issues| image:: https://img.shields.io/github/issues/fastobo/fastobo-py.svg?style=flat-square&maxAge=600
   :target: https://github.com/fastobo/fastobo-py/issues

.. |DOI| image:: https://img.shields.io/badge/doi-10.7490%2Ff1000research.1117405.1-brightgreen?style=flat-square&maxAge=31536000
   :target: https://f1000research.com/posters/8-1500

.. |Downloads| image:: https://img.shields.io/pypi/dm/fastobo?style=flat-square&color=303f9f&maxAge=86400&label=downloads
   :target: https://pepy.tech/project/fastobo

About
-----

``fastobo`` is a Rust library implementing a reliable parser for the 
`OBO file format 1.4 <https://owlcollab.github.io/oboformat/doc/GO.format.obo-1_4.html>`_.
This extension module exports idiomatic Python bindings that can be used to load, edit and
serialize ontologies in the OBO format.

Setup
-----

Run ``pip install fastobo`` in a shell to download the latest release 
from PyPi, or have a look at the :doc:`Installation page <guide/install>` to find 
other ways to install ``diced``.


Library
-------

.. toctree::
   :maxdepth: 2

   User Guide <guide/index>
   Examples <examples/index>
   API Reference <api/index>


License
-------

This library is provided under the `MIT license <https://choosealicense.com/licenses/mit/>`_.

*This project was was developed by* `Martin Larralde <https://github.com/althonos/>`_ 
*during his MSc thesis at the* 
`Lawrence Berkeley National Laboratory <https://www.lbl.gov/>`_
*in the* `BBOP team <http://berkeleybop.org/>`_.