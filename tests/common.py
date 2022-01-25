# coding: utf-8

import datetime
import unittest

import fastobo


# --- TermFrame --------------------------------------------------------------

class _TestFrame(object):

    Frame = NotImplementedError
    NameClause = NotImplementedError
    CreatedByClause = NotImplementedError

    def setUp(self):
        self.id = fastobo.id.PrefixedIdent("MS", "1000031")

    def test_init(self):
        try:
            frame = self.Frame(self.id)
        except Exception:
            self.fail("could not create frame instances")

    def test_init_iterable(self):
        try:
            frame = self.Frame(self.id, [])
        except Exception:
            self.fail("could not create frame instances")
        try:
            frame = self.Frame(self.id, [
                self.NameClause("thing"),
                self.CreatedByClause("Martin Larralde")
            ])
        except Exception:
            self.fail("could not create frame from iterable")

    def test_init_type_error(self):
        self.assertRaises(TypeError, self.Frame, 1)
        self.assertRaises(TypeError, self.Frame, [1])
        self.assertRaises(TypeError, self.Frame, ["abc"])
        self.assertRaises(TypeError, self.Frame, "abc")
        self.assertRaises(TypeError, self.Frame, self.id, 1)
        self.assertRaises(TypeError, self.Frame, self.id, [1])
        self.assertRaises(TypeError, self.Frame, self.id, ["abc"])
        self.assertRaises(TypeError, self.Frame, self.id, "abc")

    def test_append(self):
        frame = self.Frame(self.id)
        self.assertEqual(len(frame), 0)
        c1 = self.NameClause("thing")
        frame.append(c1)
        self.assertEqual(len(frame), 1)
        self.assertEqual(frame[0], c1)
        c2 = self.CreatedByClause("Martin Larralde")
        frame.append(c2)
        self.assertEqual(len(frame), 2)
        self.assertEqual(frame[0], c1)
        self.assertEqual(frame[1], c2)

    def test_reverse(self):
        c1 = self.NameClause("thing")
        c2 = self.CreatedByClause("Martin Larralde")
        frame = self.Frame(self.id, [c1, c2])
        self.assertEqual(list(frame), [c1, c2])
        frame.reverse()
        self.assertEqual(list(frame), [c2, c1])

    def test_clear(self):
        c1 = self.NameClause("thing")
        c2 = self.CreatedByClause("Martin Larralde")
        frame = self.Frame(self.id, [c1, c2])
        self.assertEqual(len(frame), 2)
        frame.clear()
        self.assertEqual(len(frame), 0)
        self.assertEqual(list(frame), [])

    def test_pop(self):
        c1 = self.NameClause("thing")
        c2 = self.CreatedByClause("Martin Larralde")
        frame = self.Frame(self.id, [c1, c2])
        self.assertEqual(len(frame), 2)
        x1 = frame.pop()
        self.assertEqual(len(frame), 1)
        self.assertEqual(x1, c2)
        x2 = frame.pop()
        self.assertEqual(len(frame), 0)
        self.assertEqual(x2, c1)
        self.assertRaises(IndexError, frame.pop)

# --- DefClause --------------------------------------------------------------

class _TestDefClause(object):

    type = NotImplementedError

    def test_repr(self):
        clause = self.type("definition")
        self.assertEqual(repr(clause), "DefClause('definition')")

        id_ = fastobo.id.PrefixedIdent('ISBN', '0321842685')
        desc = "Hacker's Delight (2nd Edition)"
        x = fastobo.xref.Xref(id_, desc)

        clause = self.type("definition", fastobo.xref.XrefList([x]))
        self.assertEqual(repr(clause), "DefClause('definition', XrefList([{!r}]))".format(x))


# --- ConsiderClause ---------------------------------------------------------

class _TestConsiderClause(object):

    type = NotImplementedError

    def setUp(self):
        self.id = fastobo.id.PrefixedIdent("MS", "1000031")
        self.id2 = fastobo.id.PrefixedIdent("MS", "1000032")

    def test_init(self):
        try:
            frame = self.type(self.id)
        except Exception:
            self.fail("could not create `ConsiderClause` instances")

    def test_init_type_error(self):
        self.assertRaises(TypeError, self.type)
        self.assertRaises(TypeError, self.type, 1)

    def test_eq(self):
        self.assertEqual(self.type(self.id), self.type(self.id))
        self.assertNotEqual(self.type(self.id), self.type(self.id2))


# --- IsObsoleteClause -------------------------------------------------------

class _TestIsObsoleteClause(object):

    type = NotImplementedError

    def test_init(self):
        try:
            frame = self.type(True)
        except Exception:
            self.fail("could not create `IsObsoleteClause` instances")

    def test_property_obsolete(self):
        c = self.type(False)
        self.assertEqual(c.obsolete, False)
        c.obsolete = True
        self.assertEqual(c.obsolete, True)

    def test_repr(self):
        self.assertEqual(repr(self.type(False)), "IsObsoleteClause(False)")
        self.assertEqual(repr(self.type(True)), "IsObsoleteClause(True)")

    def test_str(self):
        self.assertEqual(str(self.type(False)), "is_obsolete: false")
        self.assertEqual(str(self.type(True)), "is_obsolete: true")

    def test_eq(self):
        self.assertEqual(self.type(True), self.type(True))
        self.assertEqual(self.type(False), self.type(False))
        self.assertNotEqual(self.type(False), self.type(True))


# --- CreationDateClause -----------------------------------------------------

class _TestCreationDateClause(object):

    type = NotImplementedError

    def test_date(self):
        d1 = datetime.date(2021, 1, 23)
        clause = self.type(d1)
        self.assertEqual(str(clause), "creation_date: 2021-01-23")
        self.assertEqual(repr(clause), "CreationDateClause(datetime.date(2021, 1, 23))")
        self.assertEqual(clause.date, d1)
        self.assertIsInstance(clause.date, datetime.date)
        d2 = datetime.date(2021, 2, 15)
        clause.date = d2
        self.assertIsInstance(clause.date, datetime.date)

    def test_datetime(self):
        d1 = datetime.datetime(2021, 1, 23, 12)
        clause = self.type(d1)
        self.assertEqual(str(clause), "creation_date: 2021-01-23T12:00:00")
        self.assertEqual(repr(clause), "CreationDateClause(datetime.datetime(2021, 1, 23, 12, 0))")
        self.assertEqual(clause.date, d1)
        self.assertIsInstance(clause.date, datetime.datetime)
        d2 = datetime.datetime(2021, 2, 15, 12, 30, 0, tzinfo=datetime.timezone.utc)
        clause.date = d2
        self.assertEqual(str(clause), "creation_date: 2021-02-15T12:30:00Z")
        self.assertIsInstance(clause.date, datetime.datetime)
