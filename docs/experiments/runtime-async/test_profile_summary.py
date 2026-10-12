import unittest
from profile_summary import pause_summary


def sample(counts):
    return dict(gcPauseBuckets=counts + [0] * (64 - len(counts)), gcAttempts=sum(counts),
                gcFailures=0, livePayloadBytesAfterCollection=0, liveBlocksAfterCollection=0,
                allocatedPayloadBytes=8, reclaimedPayloadBytes=8, gcMaximumNanoseconds=15)


class PauseSummaryTests(unittest.TestCase):
    def test_combines_attempts_not_per_invocation_percentiles(self):
        result = pause_summary([sample([1]), sample([0, 0, 0, 99])])
        self.assertEqual(result['attempts'], 100)
        self.assertEqual(result['p50UpperBoundNanoseconds'], 15)
        self.assertEqual(result['p99UpperBoundNanoseconds'], 15)

    def test_zero_duration_and_last_bucket(self):
        self.assertEqual(pause_summary([sample([1])])['p50UpperBoundNanoseconds'], 1)
        self.assertEqual(pause_summary([sample([0] * 63 + [1])])['p99UpperBoundNanoseconds'], 2**64 - 1)

    def test_rejects_failed_inconsistent_or_unreclaimed_samples(self):
        for changes in [dict(gcFailures=1), dict(gcAttempts=7), dict(liveBlocksAfterCollection=1),
                        dict(reclaimedPayloadBytes=0), dict(gcPauseBuckets=[-1] * 64)]:
            with self.subTest(changes=changes), self.assertRaises(ValueError):
                pause_summary([dict(sample([1]), **changes)])
        with self.assertRaises(ValueError):
            pause_summary([sample([])])
