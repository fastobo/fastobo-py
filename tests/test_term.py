# coding: utf-8

import datetime
import unittest

import fastobo

# --- TermFrame --------------------------------------------------------------

class TestTermFrame(unittest.TestCase):

    type = fastobo.term.TermFrame

    def setUp(self):
        self.id = fastobo.id.PrefixedIdent("MS", "1000031")

    def test_init(self):
        try:
            frame = self.type(self.id)
        except Exception:
            self.fail("could not create `TermFrame` instances")

    def test_init_iterable(self):
        try:
            frame = self.type(self.id, [])
        except Exception:
            self.fail("could not create `TermFrame` instances")
        try:
            frame = self.type(self.id, [
                fastobo.term.NameClause("thing"),
                fastobo.term.CreatedByClause("Martin Larralde")
            ])
        except Exception:
            self.fail("could not create `TermFrame` from iterable")

    def test_init_type_error(self):
        self.assertRaises(TypeError, self.type, 1)
        self.assertRaises(TypeError, self.type, [1])
        self.assertRaises(TypeError, self.type, ["abc"])
        self.assertRaises(TypeError, self.type, "abc")
        self.assertRaises(TypeError, self.type, self.id, 1)
        self.assertRaises(TypeError, self.type, self.id, [1])
        self.assertRaises(TypeError, self.type, self.id, ["abc"])
        self.assertRaises(TypeError, self.type, self.id, "abc")


# --- DefClause ---------------------------------------------------------

class TestDefClause(unittest.TestCase):

    type = fastobo.term.DefClause

    def test_repr(self):
        clause = self.type("definition")
        self.assertEqual(repr(clause), "DefClause('definition')")

        id_ = fastobo.id.PrefixedIdent('ISBN', '0321842685')
        desc = "Hacker's Delight (2nd Edition)"
        x = fastobo.xref.Xref(id_, desc)

        clause = self.type("definition", fastobo.xref.XrefList([x]))
        self.assertEqual(repr(clause), "DefClause('definition', XrefList([{!r}]))".format(x))


# --- ConsiderClause ---------------------------------------------------------

class TestConsiderClause(unittest.TestCase):

    type = fastobo.term.ConsiderClause

    def setUp(self):
        self.id = fastobo.id.PrefixedIdent("MS", "1000031")
        self.id2 = fastobo.id.PrefixedIdent("MS", "1000032")

    def test_init(self):
        try:
            frame = self.type(self.id)
        except Exception:
            self.fail("could not create `ConsiderClause` instances")

    def test_init_type_error(self):
        self.assertRaises(TypeError, self.type)
        self.assertRaises(TypeError, self.type, 1)

    def test_eq(self):
        self.assertEqual(self.type(self.id), self.type(self.id))
        self.assertNotEqual(self.type(self.id), self.type(self.id2))


# --- IsObsoleteClause -------------------------------------------------------


class TestIsObsoleteClause(unittest.TestCase):

    type = fastobo.term.IsObsoleteClause

    def test_init(self):
        try:
            frame = self.type(True)
        except Exception:
            self.fail("could not create `IsObsoleteClause` instances")

    def test_property_obsolete(self):
        c = self.type(False)
        self.assertEqual(c.obsolete, False)
        c.obsolete = True
        self.assertEqual(c.obsolete, True)

    def test_repr(self):
        self.assertEqual(repr(self.type(False)), "IsObsoleteClause(False)")
        self.assertEqual(repr(self.type(True)), "IsObsoleteClause(True)")

    def test_str(self):
        self.assertEqual(str(self.type(False)), "is_obsolete: false")
        self.assertEqual(str(self.type(True)), "is_obsolete: true")

    def test_eq(self):
        self.assertEqual(self.type(True), self.type(True))
        self.assertEqual(self.type(False), self.type(False))
        self.assertNotEqual(self.type(False), self.type(True))
