import unittest
from summarize import counters, capacity

class SummaryTests(unittest.TestCase):
    def setUp(self):
        c=dict(base_calls=3,base_cache_hits=2,base_walks=1,newly_marked=2,payload_matches=3,mark_candidates=5,index_queries=6,index_outside=3,index_probes=4)
        self.report=dict(passed=True,fields=list(c),trials=[dict(mode='jit',count=4,exitCode=0,counters=list(c.values()))])
    def test_valid(self):
        self.assertEqual(counters(self.report)[0]['counters']['base_walks'],1)
    def test_failed(self):
        self.report['passed']=False
        with self.assertRaises(ValueError):counters(self.report)
    def test_bad_counts(self):
        self.report['trials'][0]['counters'][1]=10
        with self.assertRaises(ValueError):counters(self.report)
    def test_truncated(self):
        self.report['trials'][0]['counters'].pop()
        with self.assertRaises(ValueError):counters(self.report)
    def test_capacity_mean(self):
        rows=[dict(label=label,exitCode=0,trial=dict(requestsPerSecond=rate),cpuMicrosecondsPerResponse=rate*2)
              for label in ['a','b','c','d'] for rate in [2,4]]
        self.assertEqual(capacity(dict(passed=True,trials=rows))['a'],dict(requestsPerSecond=3,cpuMicrosecondsPerResponse=6))
        rows.pop()
        with self.assertRaises(ValueError):capacity(dict(passed=True,trials=rows))

if __name__=='__main__':unittest.main()
