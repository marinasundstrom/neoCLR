import gzip
import json
from pathlib import Path
import tempfile
import unittest
from summarize import summarize


class SummaryTests(unittest.TestCase):
    def test_recorded_cohort(self):
        base = Path(__file__).parent
        rows = summarize([base/'plain-final.json.gz', base/'json-final.json.gz'])
        self.assertEqual(sum(row['requests'] for row in rows), 4096)
        self.assertEqual(sum(row['comparedCollections'] for row in rows), 6188)

    def test_rejects_incomplete_or_failed_comparison(self):
        base = Path(__file__).parent
        for failure in ['missing', 'invalid', 'failed', 'duplicate']:
            report = json.loads(gzip.decompress((base/'plain-final.json.gz').read_bytes()))
            if failure == 'missing':
                report['trials'][0]['shadow'][0] -= 1
            elif failure == 'invalid':
                report['trials'][0]['shadow'][4] = 1
            elif failure == 'failed':
                report['passed'] = False
            else:
                report['trials'][1]['mode'] = report['trials'][0]['mode']
            with self.subTest(failure=failure), tempfile.TemporaryDirectory() as directory:
                changed = Path(directory)/'changed.json.gz'
                changed.write_bytes(gzip.compress(json.dumps(report).encode()))
                with self.assertRaises(AssertionError):
                    summarize([changed, base/'json-final.json.gz'])


if __name__ == '__main__':
    unittest.main()
