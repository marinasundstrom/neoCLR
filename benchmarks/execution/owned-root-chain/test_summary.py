import unittest
from summarize import pair,summarize

class SummaryTests(unittest.TestCase):
    def test_paired_arithmetic(self):
        rows=[dict(label=label,requestsPerSecond=rate,cpuMicrosecondsPerResponse=cpu,
                   latencyMilliseconds={'p99':1}) for label,rate,cpu in [('b',100,10),('c',125,8),('c',125,8),('b',100,10)]]
        r=pair(rows,'label','b','c')
        self.assertAlmostEqual(r['throughputGainPercent'],25)
        self.assertAlmostEqual(r['cpuReductionPercent'],20)
        with self.assertRaises(ValueError):pair(rows[:-1],'label','b','c')
        rows[0]['exitCode']=1
        with self.assertRaises(ValueError):pair(rows,'label','b','c')

    def test_reject_profile_or_failed(self):
        with self.assertRaises(ValueError):summarize({'passed':False},{'passed':True})
        with self.assertRaises(ValueError):summarize({'passed':True},{'passed':True,'profile':True})
        with self.assertRaises(ValueError):summarize({'passed':True,'trials':[]},{'passed':True,'profile':False,'trials':[]})

if __name__=='__main__':unittest.main()
