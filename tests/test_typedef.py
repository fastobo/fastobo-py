# coding: utf-8

import datetime
import unittest

import fastobo

from .common import (
    _TestFrame,
    _TestIsObsoleteClause,
    _TestDefClause,
    _TestConsiderClause,
    _TestCreationDateClause,
)

# --- TypedefFrame -----------------------------------------------------------

class TestTypedefFrame(_TestFrame, unittest.TestCase):
    Frame = fastobo.typedef.TypedefFrame
    NameClause = fastobo.typedef.NameClause
    CreatedByClause = fastobo.typedef.CreatedByClause


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

class TestCreationDateClause(_TestCreationDateClause, unittest.TestCase):
    type = fastobo.typedef.CreationDateClause
