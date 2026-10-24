import unittest
from summarize import summarize

class SummaryTests(unittest.TestCase):
    def setUp(self):
        self.report=dict(passed=True,profile=False,trials=[dict(label=label,exitCode=0,jit=dict(guestExitCode=0),
             trial=dict(requestsPerSecond=100 if label=='system' else 125),
             cpuMicrosecondsPerResponse=10 if label=='system' else 8)
             for label in ['system','arena','arena','system']])
    def test_changes(self):
        result=summarize(self.report)
        self.assertEqual(result['throughputGainPercent'],25)
        self.assertAlmostEqual(result['cpuReductionPercent'],20)
    def test_profile_rejected(self):
        self.report['profile']=True
        with self.assertRaises(ValueError):summarize(self.report)
    def test_incomplete_rejected(self):
        self.report['trials'].pop()
        with self.assertRaises(ValueError):summarize(self.report)
    def test_guest_failure_rejected(self):
        self.report['trials'][0]['jit']['guestExitCode']=3
        with self.assertRaises(ValueError):summarize(self.report)

if __name__=='__main__':unittest.main()
