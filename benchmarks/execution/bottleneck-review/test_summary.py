import copy
import unittest
from summarize import groups,peers,startup,lookup

class SummaryTests(unittest.TestCase):
    def test_exclusive_accounting(self):
        result=groups({'stack_roots':30,'collect_indexed':20,'_tlv_get_addr':10,'neoclr_value_1':15,'unknown':25})
        self.assertEqual(result['total'],100)
        self.assertEqual(sum(x['samples'] for x in result['groups'].values()),100)
        self.assertEqual(result['groups']['collector']['percent'],20)
        self.assertEqual(result['groups']['other (retained separately)']['samples'],25)

    def test_reject_negative_samples(self):
        with self.assertRaises(ValueError):groups({'x':-1})

    def test_incomplete_or_failed_startup(self):
        with self.assertRaises(ValueError):startup(dict(passed=False))
        with self.assertRaises(ValueError):startup(dict(passed=True,runs=[]))

    def test_capacity_denominator(self):
        report=dict(passed=True,trials=[])
        for name in ['neoclr-1','neoclr-4','go-1','go-4','kestrel-1','kestrel-4','node-1']:
            for _ in range(2):
                report['trials'].append(dict(name=name,completed=80,drained=10,warmup=dict(completed=100,drained=10),
                    requestsPerSecond=80,processCpuSeconds=.02,latencyMilliseconds=dict(p99=1)))
        self.assertAlmostEqual(peers(report)['neoclr-1']['cpuMicrosecondsPerResponse'],100)
        broken=copy.deepcopy(report);broken['trials'].pop()
        with self.assertRaises(ValueError):peers(broken)

    def test_lookup_increment_and_rejection(self):
        report=dict(passed=True,fields=['words'],trials=[
            dict(mode=mode,count=count,counters=[100+count*20],exitCode=0)
            for mode in ['aot','jit'] for count in [4,12]])
        self.assertEqual(lookup(report),{'aot':{'words':20},'jit':{'words':20}})
        report['trials'][0]['counters']=[]
        with self.assertRaises(ValueError):lookup(report)

if __name__=='__main__':unittest.main()
