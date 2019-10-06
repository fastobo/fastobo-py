# coding: utf-8

import datetime
import unittest

import fastobo


# -- OboDoc ------------------------------------------------------------------

class TestOboDoc(unittest.TestCase):

    type = fastobo.doc.OboDoc

    def setUp(self):
        self.header = fastobo.header.HeaderFrame([
            fastobo.header.FormatVersionClause("1.4"),
            fastobo.header.SavedByClause("Martin Larralde"),
        ])
        self.entities = [
            fastobo.term.TermFrame(fastobo.id.PrefixedIdent("MS", "1000031")),
            fastobo.typedef.TypedefFrame(fastobo.id.UnprefixedIdent("part_of"))
        ]

    def test_init(self):
        try:
            doc = self.type()
        except Exception:
            self.fail("could not create `OboDoc` instances")
        self.assertEqual(len(doc.header), 0)
        self.assertEqual(len(doc), 0)

    def test_init_header(self):
        try:
            doc = self.type(self.header)
        except Exception:
            self.fail("could not create `OboDoc` instances with a header")
        self.assertEqual(len(doc.header), 2)
        self.assertEqual(doc.header[0], self.header[0])
        self.assertEqual(doc.header[1], self.header[1])
        self.assertEqual(len(doc), 0)

    def test_init_entities(self):
        try:
            doc = self.type(entities=self.entities)
        except Exception:
            self.fail("could not create `OboDoc` instances with a header")
        self.assertEqual(len(doc.header), 0)
        self.assertEqual(len(doc), 2)
        self.assertEqual(doc[0], self.entities[0])
        self.assertEqual(doc[1], self.entities[1])

    def test_init_type_error(self):
        self.assertRaises(TypeError, self.type, 1)
        self.assertRaises(TypeError, self.type, [1])
        self.assertRaises(TypeError, self.type, ["abc"])
        self.assertRaises(TypeError, self.type, "abc")
        self.assertRaises(TypeError, self.type, self.header, 1)
        self.assertRaises(TypeError, self.type, self.header, [1])
        self.assertRaises(TypeError, self.type, self.header, ["abc"])
        self.assertRaises(TypeError, self.type, self.header, "abc")
        self.assertRaises(TypeError, self.type, 1, self.entities)
        self.assertRaises(TypeError, self.type, [1], self.entities)
        self.assertRaises(TypeError, self.type, ["abc"], self.entities)
        self.assertRaises(TypeError, self.type, "abc", self.entities)
