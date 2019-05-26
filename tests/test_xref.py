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

    def test_init(self):
        xref1 = fastobo.xref.Xref(
            fastobo.id.UnprefixedIdent("fastobo")
        )
        xref2 = fastobo.xref.Xref(
            fastobo.id.PrefixedIdent('ISBN', '0321842685')
        )

        try:
            xref = self.type()
        except Exception:
            self.fail("could not create `XrefList` instance without argument")
        try:
            xref = self.type([xref1, xref2])
        except Exception:
            self.fail("could not create `XrefList` instance from list")
        try:
            xref = self.type(iter([xref1, xref2]))
        except Exception:
            self.fail("could not create `XrefList` instance from iterator")

    def test_init_type_error(self):
        # Errors on an iterator of type != Xref
        self.assertRaises(TypeError, self.type, "abc")
        self.assertRaises(TypeError, self.type, ["abc", "def"])

    def test_str(self):

        id = fastobo.id.PrefixedIdent('ISBN', '0321842685')
        desc = "Hacker's Delight (2nd Edition)"
        xref1 = fastobo.xref.Xref(id, desc)

        id = fastobo.id.UnprefixedIdent("fastobo")
        xref2 = fastobo.xref.Xref(id)

        self.assertEqual(
            str(self.type()),
            "[]"
        )
        self.assertEqual(
            str(self.type([xref1])),
            '[ISBN:0321842685 "Hacker\'s Delight (2nd Edition)"]'
        )
        self.assertEqual(
            str(self.type([xref1, xref2])),
            '[ISBN:0321842685 "Hacker\'s Delight (2nd Edition)", fastobo]'
        )
