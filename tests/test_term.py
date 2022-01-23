# coding: utf-8

import datetime
import unittest

import fastobo

from .common import (
    _TestFrame,
    _TestIsObsoleteClause,
    _TestDefClause,
    _TestConsiderClause,
    _TestIsObsoleteClause,
)

# --- TermFrame --------------------------------------------------------------

class TestTermFrame(_TestFrame, unittest.TestCase):
    type = fastobo.term.TermFrame


# --- DefClause --------------------------------------------------------------

class TestDefClause(_TestDefClause, unittest.TestCase):
    type = fastobo.term.DefClause


# --- ConsiderClause ---------------------------------------------------------

class TestConsiderClause(_TestConsiderClause, unittest.TestCase):
    type = fastobo.term.ConsiderClause


# --- IsObsoleteClause -------------------------------------------------------

class TestIsObsoleteClause(_TestIsObsoleteClause, unittest.TestCase):
    type = fastobo.term.IsObsoleteClause


# --- CreationDateClause -----------------------------------------------------

class TestIsObsoleteClause(_TestIsObsoleteClause, unittest.TestCase):
    type = fastobo.term.CreationDateClause
