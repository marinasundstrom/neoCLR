import copy
import gzip
import json
from pathlib import Path
import unittest
from summarize import summarize


class SummaryTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.report = json.loads(gzip.decompress((Path(__file__).parent/'capacity.json.gz').read_bytes()))

    def test_complete_capacity(self):
        result = summarize(self.report)
        self.assertAlmostEqual(result['jit']['owned']['requestsPerSecond'], 1885.0)

    def test_rejects_profile_cohort(self):
        profile = json.loads(gzip.decompress((Path(__file__).parent/'profile.json.gz').read_bytes()))
        with self.assertRaises(ValueError):
            summarize(profile)

    def test_rejects_missing_or_failed_trials(self):
        report = copy.deepcopy(self.report)
        report['passed'] = False
        with self.assertRaises(ValueError):
            summarize(report)
        report['passed'] = True
        report['trials'].pop()
        with self.assertRaises(ValueError):
            summarize(report)

    def test_checks_cpu_denominator(self):
        report = copy.deepcopy(self.report)
        report['trials'][0]['windowCpuMicrosecondsPerResponse'] *= 2
        with self.assertRaises(ValueError):
            summarize(report)


if __name__ == '__main__':
    unittest.main()
