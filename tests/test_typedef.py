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

class TestTypedefFrame(_TestFrame, unittest.TestCase):
    type = fastobo.typedef.TypedefFrame


# --- DefClause --------------------------------------------------------------

class TestDefClause(_TestDefClause, unittest.TestCase):
    type = fastobo.typedef.DefClause


# --- ConsiderClause ---------------------------------------------------------

class TestConsiderClause(_TestConsiderClause, unittest.TestCase):
    type = fastobo.typedef.ConsiderClause


# --- IsObsoleteClause -------------------------------------------------------

class TestIsObsoleteClause(_TestIsObsoleteClause, unittest.TestCase):
    type = fastobo.typedef.IsObsoleteClause


# --- CreationDateClause -----------------------------------------------------

class TestIsObsoleteClause(_TestIsObsoleteClause, unittest.TestCase):
    type = fastobo.typedef.CreationDateClause
