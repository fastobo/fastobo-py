from . import (
    test_doc,
    test_doctests,
    test_fastobo,
    test_header,
    test_id,
    test_pv,
    test_term,
    test_typedef,
    test_xref
)

def load_tests(loader, suite, pattern):
    suite.addTests(loader.loadTestsFromModule(test_doc))
    suite.addTests(loader.loadTestsFromModule(test_doctests))
    suite.addTests(loader.loadTestsFromModule(test_fastobo))
    suite.addTests(loader.loadTestsFromModule(test_header))
    suite.addTests(loader.loadTestsFromModule(test_id))
    suite.addTests(loader.loadTestsFromModule(test_pv))
    suite.addTests(loader.loadTestsFromModule(test_term))
    suite.addTests(loader.loadTestsFromModule(test_typedef))
    suite.addTests(loader.loadTestsFromModule(test_xref))
    return suite