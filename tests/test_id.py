# coding: utf-8

import datetime
import unittest

import fastobo


class _TestBaseIdent(object):

    type = NotImplemented

    def test_init_type_error(self):
        self.assertRaises(TypeError, self.type, 123)
        self.assertRaises(TypeError, self.type, [])


class TestUnprefixedIdent(_TestBaseIdent, unittest.TestCase):

    type = fastobo.id.UnprefixedIdent

    def test_init(self):
        try:
            self.type('created_by')
        except Exception:
            self.fail("could not instantiate `UnprefixedIdent`")

    def test_eq(self):
        ident = self.type('derived_from')
        self.assertEqual(ident, self.type('derived_from'))
        self.assertNotEqual(ident, self.type('has_elements_from'))
        self.assertNotEqual(ident, 123)

    def test_cmp(self):
        ident = self.type('derived_from')
        self.assertLessEqual(ident, self.type('derived_from'))
        self.assertLessEqual(ident, self.type('has_elements_from'))
        self.assertRaises(TypeError, ident.__lt__, 123)
        self.assertRaises(TypeError, ident.__le__, 123)
        self.assertRaises(TypeError, ident.__gt__, 123)
        self.assertRaises(TypeError, ident.__ge__, 123)

    def test_hash(self):
        ident = self.type('derived_from')
        self.assertEqual(hash(ident), hash(self.type('derived_from')))
        self.assertNotEqual(hash(ident), hash(self.type('has_elements_from')))


class TestPrefixedIdent(_TestBaseIdent, unittest.TestCase):

    type = fastobo.id.PrefixedIdent

    def test_init(self):
        try:
            self.type('GO', '0070412')
        except Exception:
            self.fail("could not instantiate `PrefixedIdent`")

    def test_init_type_error(self):
        self.assertRaises(TypeError, self.type, "GO", 123)
        self.assertRaises(TypeError, self.type, "GO", [])
        self.assertRaises(TypeError, self.type, 123, "0070412")
        self.assertRaises(TypeError, self.type, [], "0070412")

    def test_hash(self):
        ident = self.type("GO", "0070412")
        self.assertEqual(hash(ident), hash(self.type("GO", "0070412")))
        self.assertNotEqual(hash(ident), hash(self.type("GO", "0070413")))


class TestUrl(_TestBaseIdent, unittest.TestCase):

    type = fastobo.id.Url

    def test_init(self):
        try:
            self.type('http://purl.obolibrary.org/obo/GO_0070412')
        except Exception:
            self.fail("could not instantiate `Url`")

    def test_init_type_error(self):
        self.assertRaises(TypeError, self.type, 123)
        self.assertRaises(TypeError, self.type, [])

    def test_init_value_error(self):
        self.assertRaises(ValueError, self.type, "not a URL at all")

    def test_eq(self):
        url = self.type('http://purl.obolibrary.org/obo/GO_0070412')
        self.assertEqual(url, self.type('http://purl.obolibrary.org/obo/GO_0070412'))
        self.assertNotEqual(url, self.type('http://purl.obolibrary.org/obo/GO_0070413'))
        self.assertNotEqual(url, 123)

    def test_cmp(self):
        url = self.type('http://purl.obolibrary.org/obo/GO_0070412')
        self.assertLess(url, self.type('http://purl.obolibrary.org/obo/GO_0070413'))
        self.assertRaises(TypeError, url.__lt__, 'http://purl.obolibrary.org/obo/GO_0070413')

    def test_hash(self):
        url = self.type('http://purl.obolibrary.org/obo/GO_0070412')
        self.assertEqual(hash(url), hash(self.type('http://purl.obolibrary.org/obo/GO_0070412')))
        self.assertNotEqual(hash(url), hash(self.type('http://purl.obolibrary.org/obo/GO_0070413')))
