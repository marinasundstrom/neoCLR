import unittest
from diagnostic_session import gc_session, METRICS


def snapshot():
    return dict(version=1, pid=123, instance='a'*32, capabilities=['gc-snapshot'],
                scope='runtime-owner-thread', available=True, fresh=True, sequence=1,
                timestampNanoseconds=2**53+1, ageNanoseconds=1,
                gcPauseBuckets=[0]*64, **{name: 2**53+1 for name in METRICS})


class SessionTests(unittest.TestCase):
    def test_lossless_values_and_availability(self):
        absent = snapshot()
        absent.update(available=False, fresh=False, sequence=0, timestampNanoseconds=0)
        fresh = snapshot()
        cached = dict(fresh, fresh=False, ageNanoseconds=100)
        s = gc_session([absent, fresh, cached])
        self.assertIsNone(s['observations'][0]['metrics'])
        self.assertEqual('9007199254740993', s['observations'][1]['metrics']['allocations'])
        self.assertEqual('cached', s['observations'][2]['availability'])
        self.assertFalse(s['capabilities']['heapSnapshots'])

    def test_incompatible_or_corrupt_input_rejected(self):
        for change in [dict(version=2), dict(pid=0), dict(instance='wrong'),
                       dict(scope='process'), dict(allocations=-1), dict(allocations=True),
                       dict(allocations=2**64), dict(gcPauseBuckets=[0]),
                       dict(available=False), dict(fresh=1)]:
            with self.subTest(change=change), self.assertRaises(ValueError):
                gc_session([dict(snapshot(), **change)])

    def test_identity_cache_and_reset(self):
        for change in [dict(instance='b'*32), dict(sequence=0),
                       dict(timestampNanoseconds=2**53+2),
                       dict(allocations=2**53+2),
                       dict(sequence=2, timestampNanoseconds=2**53+2, allocations=0)]:
            with self.subTest(change=change), self.assertRaises(ValueError):
                gc_session([snapshot(), dict(snapshot(), **change)])

    def test_new_snapshot_allows_live_gauge_to_fall(self):
        second = dict(snapshot(), sequence=2, timestampNanoseconds=2**53+2,
                      livePayloadBytesAfterCollection=0)
        self.assertEqual('0', gc_session([snapshot(), second])['observations'][1]['metrics']['livePayloadBytesAfterCollection'])

    def test_bounds(self):
        for rows in ([], [snapshot()]*10001):
            with self.assertRaises(ValueError): gc_session(rows)


if __name__ == '__main__': unittest.main()
