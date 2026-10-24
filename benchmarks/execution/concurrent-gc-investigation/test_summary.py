import copy,unittest
from pathlib import Path
from summarize import read,summarize
class Summary(unittest.TestCase):
 def setUp(self):
  p=Path(__file__).parent;self.inputs=[read(p/(n+'.json.gz')) for n in ['plain','json','phases-plain','phases-json']]
 def test_real_counts(self):
  s=summarize(*self.inputs)
  for mode in ['aot','jit']:
   self.assertGreater(s['plain'][mode]['phaseSharePercent']['trace'],60)
   self.assertGreater(s['json'][mode]['phaseMeanMicrosecondsPerRequest']['trace'],s['plain'][mode]['phaseMeanMicrosecondsPerRequest']['trace'])
 def test_failed_execution(self):
  self.inputs[0]['trials'][0]['gc'][1]=1
  with self.assertRaises(AssertionError):summarize(*self.inputs)
 def test_incomplete_cohort(self):
  self.inputs[1]['trials'].pop()
  with self.assertRaises(AssertionError):summarize(*self.inputs)
 def test_unmatched_cycle_counts(self):
  self.inputs[2]['trials'][0]['gc'][0]+=1
  with self.assertRaises(AssertionError):summarize(*self.inputs)
if __name__=='__main__':unittest.main()
