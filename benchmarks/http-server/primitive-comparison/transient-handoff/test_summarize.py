#!/usr/bin/env python3
import gzip
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

BASE = Path(__file__).resolve().parent

class SummaryTests(unittest.TestCase):
    def run_summary(self, change=None):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            reports = {}
            for option, name in [('plain-small','paced-plain8'), ('plain-large','paced-plain24'),
                                 ('json-small','paced-json8'), ('json-large','paced-json24'), ('corpus','corpus')]:
                reports[option] = json.loads(gzip.decompress((BASE/(name+'.json.gz')).read_bytes()))
            if change: change(reports)
            cmd = [sys.executable, str(BASE/'summarize.py')]
            for option, report in reports.items():
                path = root/(option+'.json')
                path.write_text(json.dumps(report))
                cmd += ['--'+option, str(path)]
            output = root/'result.json'
            result = subprocess.run(cmd+['--output',str(output)], capture_output=True, text=True)
            return result.returncode, json.loads(output.read_text()) if output.exists() else None

    def test_units_differences_and_exclusion(self):
        code, result = self.run_summary()
        self.assertEqual(code, 0)
        self.assertEqual(result['http']['plain']['counts']['allocations'], 208)
        self.assertEqual(result['http']['json']['counts']['boundaries'], 5535)
        self.assertEqual(result['ordinary'][0]['counts']['allocations'], 28.5)
        self.assertFalse(result['corpusComplete'])
        self.assertEqual(len(result['excluded']), 1)

    def test_reject_nonincreasing_response_count(self):
        def change(r): r['plain-large']['responses'] = r['plain-small']['responses']
        self.assertNotEqual(self.run_summary(change)[0], 0)

    def test_reject_changed_diagnostic(self):
        def change(r):
            key = next(k for k in r['json-large']['inputs'] if k.endswith('/diagnostic-gc.c'))
            r['json-large']['inputs'][key] = 'changed'
        self.assertNotEqual(self.run_summary(change)[0], 0)

    def test_reject_changed_object_within_pair(self):
        def change(r):
            key = next(k for k in r['plain-large']['inputs'] if k.endswith('.o'))
            r['plain-large']['inputs'][key] = 'changed'
        self.assertNotEqual(self.run_summary(change)[0], 0)

if __name__ == '__main__': unittest.main()
