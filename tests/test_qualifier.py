# coding: utf-8

import datetime
import unittest

import fastobo


class TestQualifier(unittest.TestCase):

    type = fastobo.qual.Qualifier

    def test_init(self):
        key = fastobo.id.UnprefixedIdent("source")
        value = "ISBN:0321842685"
        try:
            qual = self.type(key, value)
        except Exception:
            self.fail("could not create `Qualifier` instance")

    def test_init_type_error(self):
        key = fastobo.id.UnprefixedIdent("source")
        value = "ISBN:0321842685"
        self.assertRaises(TypeError, self.type, 1)
        self.assertRaises(TypeError, self.type, 1, value)
        self.assertRaises(TypeError, self.type, key, 1)

    def test_str(self):
        key = fastobo.id.UnprefixedIdent("source")
        value = "ISBN:0321842685"
        self.assertEqual(str(self.type(key, value)), 'source="ISBN:0321842685"')

    def test_eq(self):
        i1 = fastobo.id.UnprefixedIdent('a')
        i2 = fastobo.id.UnprefixedIdent('b')
        q1 = self.type(i1, "test")
        self.assertEqual(q1, q1)
        q2 = self.type(i1, "test")
        self.assertIsNot(q1, q2)
        self.assertEqual(q1, q2)
        q3 = self.type(i2, "test")
        self.assertNotEqual(q1, q3)
        q4 = self.type(i1, "test2")
        self.assertNotEqual(q1, q4)


class TestQualifierList(unittest.TestCase):

    type = fastobo.qual.QualifierList

    def setUp(self):
        self.q1 = fastobo.qual.Qualifier(fastobo.id.UnprefixedIdent('source'), "ISBN:0321842685")
        self.q2 = fastobo.qual.Qualifier(fastobo.id.UnprefixedIdent("minCardinality"), "2")

    def test_init(self):
        try:
            ql = self.type()
        except Exception:
            self.fail("could not create `QualifierList` instance without argument")
        try:
            ql = self.type([self.q1, self.q2])
        except Exception:
            self.fail("could not create `QualifierList` instance from list")
        try:
            ql = self.type(iter([self.q1, self.q2]))
        except Exception:
            self.fail("could not create `QualifierList` instance from iterator")

    def test_init_type_error(self):
        # Errors on an iterator of type != Qualifier
        self.assertRaises(TypeError, self.type, "abc")
        self.assertRaises(TypeError, self.type, ["abc", "def"])

    def test_str(self):
        q1, q2 = self.q1, self.q2
        self.assertEqual(str(self.type()), "{}")
        self.assertEqual(str(self.type([q1])), '{source="ISBN:0321842685"}')
        self.assertEqual(str(self.type([q1, q2])), '{source="ISBN:0321842685", minCardinality="2"}')

    def test_append(self):
        q1, q2 = self.q1, self.q2
        l = self.type()
        self.assertEqual(len(l), 0)
        l.append(q1)
        self.assertEqual(len(l), 1)
        self.assertEqual(l[0], q1)
        l.append(q2)
        self.assertEqual(len(l), 2)
        self.assertEqual(l[0], q1)
        self.assertEqual(l[1], q2)

    def test_contains(self):
        q1, q2 = self.q1, self.q2
        l1 = self.type()
        self.assertNotIn(q1, l1)
        self.assertNotIn(q2, l1)
        l2 = self.type([q1])
        self.assertIn(q1, l2)
        self.assertNotIn(q2, l2)
        l3 = self.type([q1, q2])
        self.assertIn(q1, l3)
        self.assertIn(q2, l3)

    def test_repr(self):
        q1, q2 = self.q1, self.q2
        self.assertEqual( repr(self.type()), "QualifierList()" )
        self.assertEqual( repr(self.type([q1])), "QualifierList([{!r}])".format(q1) )
        self.assertEqual( repr(self.type([q1, q2])), "QualifierList([{!r}, {!r}])".format(q1, q2) )
