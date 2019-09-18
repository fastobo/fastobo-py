# coding: utf-8

import os
import unittest

import fastobo

MS = os.path.realpath(os.path.join(__file__, "..", "data", "ms.obo"))


class TestLoad(unittest.TestCase):

    def test_file_not_found(self):
        self.assertRaises(FileNotFoundError, fastobo.load, "abcdef")

    def test_type_error(self):
        self.assertRaises(TypeError, fastobo.load, 1)
        self.assertRaises(TypeError, fastobo.load, [])

        with open(MS) as f:
            self.assertRaises(TypeError, fastobo.load, f)

    def test_error_propagation(self):

        def read(x):
            if x == 0:
                return b''
            raise RuntimeError(x)

        with open(MS, 'rb') as f:
            f.read = read
            self.assertRaises(RuntimeError, fastobo.load, f)

    def test_syntax_error(self):
        self.assertRaises(SyntaxError, fastobo.loads, "hello there")


class TestIter(unittest.TestCase):

    def test_file_not_found(self):
        self.assertRaises(FileNotFoundError, fastobo.iter, "abcdef")

    def test_type_error(self):
        self.assertRaises(TypeError, fastobo.iter, 1)
        self.assertRaises(TypeError, fastobo.iter, [])

        with open(MS) as f:
            self.assertRaises(TypeError, fastobo.iter, f)
