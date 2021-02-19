# coding: utf-8

import datetime
import unittest

import fastobo


class TestXref(unittest.TestCase):

    type = fastobo.xref.Xref

    def test_init(self):
        id = fastobo.id.PrefixedIdent('ISBN', '0321842685')
        try:
            xref = self.type(id)
        except Exception:
            self.fail("could not create `Xref` instance without description")
        try:
            xref = self.type(id, "Hacker's Delight (2nd Edition)")
        except Exception:
            self.fail("could not create `Xref` instance with description")

    def test_init_type_error(self):
        id = fastobo.id.PrefixedIdent('ISBN', '0321842685')
        desc = "Hacker's Delight (2nd Edition)"
        self.assertRaises(TypeError, self.type, 1)
        self.assertRaises(TypeError, self.type, 1, desc)
        self.assertRaises(TypeError, self.type, id, 1)

    def test_str(self):
        id = fastobo.id.PrefixedIdent('ISBN', '0321842685')
        desc = "Hacker's Delight (2nd Edition)"
        self.assertEqual(str(self.type(id)), "ISBN:0321842685")
        self.assertEqual(
            str(self.type(id, desc)),
            'ISBN:0321842685 "Hacker\'s Delight (2nd Edition)"'
        )


class TestXrefList(unittest.TestCase):

    type = fastobo.xref.XrefList

    def setUp(self):
        id = fastobo.id.PrefixedIdent('ISBN', '0321842685')
        desc = "Hacker's Delight (2nd Edition)"
        self.x1 = fastobo.xref.Xref(id, desc)
        self.x2 = fastobo.xref.Xref(fastobo.id.UnprefixedIdent("fastobo"))

    def test_init(self):
        try:
            xref = self.type()
        except Exception:
            self.fail("could not create `XrefList` instance without argument")
        try:
            xref = self.type([self.x1, self.x2])
        except Exception:
            self.fail("could not create `XrefList` instance from list")
        try:
            xref = self.type(iter([self.x1, self.x2]))
        except Exception:
            self.fail("could not create `XrefList` instance from iterator")

    def test_init_type_error(self):
        # Errors on an iterator of type != Xref
        self.assertRaises(TypeError, self.type, "abc")
        self.assertRaises(TypeError, self.type, ["abc", "def"])

    def test_str(self):
        x1, x2 = self.x1, self.x2
        self.assertEqual(str(self.type()), "[]")
        self.assertEqual(str(self.type([x1])), '[{}]'.format(x1))
        self.assertEqual(str(self.type([x1, x2])), '[{}, {}]'.format(x1, x2))

    def test_contains(self):
        x1, x2 = self.x1, self.x2
        l1 = self.type()
        self.assertNotIn(x1, l1)
        self.assertNotIn(x2, l1)
        l2 = self.type([x1])
        self.assertIn(x1, l2)
        self.assertNotIn(x2, l2)
        l3 = self.type([x1, x2])
        self.assertIn(x1, l3)
        self.assertIn(x2, l3)

    def test_repr(self):
        x1, x2 = self.x1, self.x2
        self.assertEqual( repr(self.type()), "XrefList()" )
        self.assertEqual( repr(self.type([x1])), "XrefList([{!r}])".format(x1) )
        self.assertEqual( repr(self.type([x1, x2])), "XrefList([{!r}, {!r}])".format(x1, x2) )
