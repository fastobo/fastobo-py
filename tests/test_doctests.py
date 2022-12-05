# coding: utf-8
"""Test doctest contained tests in every file of the module.
"""

import os
import sys
import datetime
import doctest
import warnings
import pprint
import textwrap
import types

import fastobo


def _load_tests_from_module(tests, module, globs, setUp=None, tearDown=None):
    """Load tests from module, iterating through submodules"""

    module.__test__ = {}
    for attr in (getattr(module, x) for x in dir(module) if not x.startswith('_')):
        if isinstance(attr, types.ModuleType):
            _load_tests_from_module(tests, attr, globs, setUp, tearDown)
        else:
            module.__test__[attr.__name__] = attr

    tests.addTests(doctest.DocTestSuite(
        module,
        globs=globs,
        setUp=setUp,
        tearDown=tearDown,
        optionflags=doctest.ELLIPSIS,
    ))

    return tests


def load_tests(loader, tests, ignore):
    """load_test function used by unittest to find the doctests"""

    _current_cwd = os.getcwd()

    def setUp(self):
        warnings.simplefilter("ignore")
        os.chdir(os.path.realpath(os.path.join(__file__, "..", "data")))

    def tearDown(self):
        os.chdir(_current_cwd)
        warnings.simplefilter(warnings.defaultaction)

    globs = {
        "fastobo": fastobo,
        "datetime": datetime,
        "textwrap": textwrap,
        "pprint": pprint.pprint,
        "ms": fastobo.load(os.path.realpath(
            os.path.join(__file__, "..", "data", "ms.obo")
        )),
    }

    if not sys.argv[0].endswith('green'):
        tests = _load_tests_from_module(tests, fastobo, globs, setUp, tearDown)
    return tests


def setUpModule():
    warnings.simplefilter('ignore')


def tearDownModule():
    warnings.simplefilter(warnings.defaultaction)
