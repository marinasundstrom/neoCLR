import copy,gzip,json,unittest
from pathlib import Path
from summarize import summarize
class SummaryTests(unittest.TestCase):
    def report(self,name='plain'):
        return json.loads(gzip.decompress(Path(__file__).with_name(name+'.json.gz').read_bytes()))
    def test_request_phase_counts(self):
        for sample,publications,allocations in [('plain',3,208),('json',5,457)]:
            for row in summarize(self.report(sample)).values():
                self.assertEqual(row['triggersPerRequest']['publication'],publications)
                self.assertEqual(row['triggersPerRequest']['allocation'],0)
                self.assertEqual(row['gcPerRequest']['allocations'],allocations)
    def test_rejects_missing_and_duplicate_rows(self):
        for mutation in ['missing','duplicate']:
            r=self.report()
            if mutation=='missing':r['trials'].pop()
            else:r['trials'][1]=copy.deepcopy(r['trials'][0])
            with self.assertRaises(ValueError):summarize(r)
    def test_rejects_unclassified_and_failed_entries(self):
        for failure in [False,True]:
            r=self.report()
            if failure:r['trials'][0]['failures'][0]=1
            else:r['trials'][0]['gc'][0]+=1
            with self.assertRaises(ValueError):summarize(r)
    def test_rejects_truncated_counter_arrays(self):
        r=self.report();r['trials'][0]['requests'].pop()
        with self.assertRaises(ValueError):summarize(r)
