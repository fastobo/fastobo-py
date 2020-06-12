# coding: utf-8

import os
import unittest

import fastobo

MS = os.path.realpath(os.path.join(__file__, "..", "data", "ms.obo"))
MS_FRAMES = 2941

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

    def test_threading_single(self):
        doc = fastobo.load(MS, threads=1)
        self.assertEqual(len(doc), MS_FRAMES)

        with open(MS, 'rb') as f:
            doc = fastobo.load(f, threads=1)
            self.assertEqual(len(doc), MS_FRAMES)

    def test_threading_explicit(self):
        doc = fastobo.load(MS, threads=4)
        self.assertEqual(len(doc), MS_FRAMES)

        with open(MS, 'rb') as f:
            doc = fastobo.load(f, threads=4)
            self.assertEqual(len(doc), MS_FRAMES)

    def test_threading_detect(self):
        doc = fastobo.load(MS, threads=0)
        self.assertEqual(len(doc), MS_FRAMES)

        with open(MS, 'rb') as f:
            doc = fastobo.load(f, threads=0)
            self.assertEqual(len(doc), MS_FRAMES)

    def test_threading_invalid(self):
        self.assertRaises(ValueError, fastobo.load, MS, threads=-1)


class TestIter(unittest.TestCase):

    def test_file_not_found(self):
        self.assertRaises(FileNotFoundError, fastobo.iter, "abcdef")

    def test_type_error(self):
        self.assertRaises(TypeError, fastobo.iter, 1)
        self.assertRaises(TypeError, fastobo.iter, [])

        with open(MS) as f:
            self.assertRaises(TypeError, fastobo.iter, f)

    def test_threading_single(self):
        frame_count = sum(1 for _ in fastobo.iter(MS, threads=1))
        self.assertEqual(frame_count, MS_FRAMES)

        with open(MS, 'rb') as f:
            frame_count = sum(1 for _ in fastobo.iter(f, threads=1))
            self.assertEqual(frame_count, MS_FRAMES)

    def test_threading_explicit(self):
        frame_count = sum(1 for _ in fastobo.iter(MS, threads=4))
        self.assertEqual(frame_count, MS_FRAMES)

        with open(MS, 'rb') as f:
            frame_count = sum(1 for _ in fastobo.iter(f, threads=4))
            self.assertEqual(frame_count, MS_FRAMES)

    def test_threading_detect(self):
        frame_count = sum(1 for _ in fastobo.iter(MS, threads=0))
        self.assertEqual(frame_count, MS_FRAMES)

        with open(MS, 'rb') as f:
            frame_count = sum(1 for _ in fastobo.iter(f, threads=0))
            self.assertEqual(frame_count, MS_FRAMES)

    def test_threading_invalid(self):
        self.assertRaises(ValueError, fastobo.iter, MS, threads=-1)

class TestLoads(unittest.TestCase):

    @classmethod
    def setUpClass(cls):
        with open(MS, 'r') as f:
            cls.text = f.read()

    def test_threading_single(self):
        doc = fastobo.loads(self.text, threads=1)
        self.assertEqual(len(doc), MS_FRAMES)

    def test_threading_explicit(self):
        doc = fastobo.loads(self.text, threads=4)
        self.assertEqual(len(doc), MS_FRAMES)

    def test_threading_detect(self):
        doc = fastobo.loads(self.text, threads=0)
        self.assertEqual(len(doc), MS_FRAMES)

    def test_threading_invalid(self):
        self.assertRaises(ValueError, fastobo.loads, self.text, threads=-1)
