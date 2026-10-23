import copy
from pathlib import Path
import unittest
from summarize import read, summarize


class SummaryTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        root = Path(__file__).parent
        cls.ordinary = read(root / 'ordinary.json.gz')
        cls.http = read(root / 'http.json.gz')

    def test_separates_startup_and_execution(self):
        result = summarize(self.ordinary, self.http)
        jit = next(x for x in result['ordinary'] if x['mode'] == 'jit')
        self.assertGreater(jit['selectionAndVerificationSeconds'], jit['invocationSeconds'])
        interpreter = next(x for x in result['ordinary'] if x['mode'] == 'interpreter')
        self.assertNotIn('invocationSeconds', interpreter)

    def test_excludes_diagnostics(self):
        changed = copy.deepcopy(self.ordinary)
        for case in changed['cases']:
            for row in case['runs']:
                if row['diagnostic']:
                    row['wallSeconds'] = 1e9
                    row['processCpuSeconds'] = 1e9
        self.assertEqual(summarize(self.ordinary, self.http), summarize(changed, self.http))

    def test_rejects_incomplete_cohort(self):
        changed = copy.deepcopy(self.http)
        changed['passed'] = False
        with self.assertRaises(ValueError):
            summarize(self.ordinary, changed)
        changed['passed'] = True
        changed['trials'].pop()
        with self.assertRaises(ValueError):
            summarize(self.ordinary, changed)

    def test_cpu_window_units_and_denominator(self):
        for row in self.http['trials']:
            responses = sum(x['completed'] + x['drained'] for x in [row['warmup'], row['trial']])
            self.assertAlmostEqual(row['windowCpuMicrosecondsPerResponse'],
                                   row['windowCpuSeconds'] * 1e6 / responses)


if __name__ == '__main__':
    unittest.main()
