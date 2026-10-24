import unittest
from summarize import summarize


class SummaryTests(unittest.TestCase):
    def setUp(self):
        self.report = dict(passed=True, trials=[
            dict(label=label, exitCode=0, trial=dict(requestsPerSecond=rate),
                 cpuMicrosecondsPerResponse=100/rate)
            for label in ['baseline', 'long-range', 'dynamic', 'jit']
            for rate in [2, 4]])

    def test_means_and_range(self):
        self.assertEqual(summarize(self.report)['jit'], dict(
            requestsPerSecond=3, rateRange=[2, 4], cpuMicrosecondsPerResponse=37.5))

    def test_incomplete(self):
        self.report['passed'] = False
        with self.assertRaises(ValueError):
            summarize(self.report)

    def test_failed_server(self):
        self.report['trials'][0]['exitCode'] = 1
        with self.assertRaises(ValueError):
            summarize(self.report)

    def test_missing_trial(self):
        self.report['trials'].pop()
        with self.assertRaises(ValueError):
            summarize(self.report)


if __name__ == '__main__':
    unittest.main()
