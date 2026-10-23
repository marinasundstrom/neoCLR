import copy
import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location('profile_summary', Path(__file__).with_name('profile-summary.py'))
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class ProfileSummaryTests(unittest.TestCase):
    def setUp(self):
        self.small = dict(passed=True, paced=True, json=False, inputs={'/a/app.o': 'same'},
                          responses=[1] * 8, boundaries=100, detail={'ancestorFrames': 200})
        self.large = copy.deepcopy(self.small)
        self.large.update(responses=[1] * 24, boundaries=132, detail={'ancestorFrames': 296})

    def test_subtracts_fixed_cost(self):
        self.assertEqual(module.summarize(self.small, self.large),
                         dict(ancestorFrames=6, boundaries=2, framesPerBoundary=3))

    def test_rejects_changed_object(self):
        self.large['inputs']['/a/app.o'] = 'different'
        with self.assertRaises(AssertionError):
            module.summarize(self.small, self.large)

    def test_rejects_nonincreasing_responses(self):
        self.large['responses'] = self.small['responses']
        with self.assertRaises(AssertionError):
            module.summarize(self.small, self.large)

    def test_rejects_failed_run(self):
        self.large['passed'] = False
        with self.assertRaises(AssertionError):
            module.summarize(self.small, self.large)


if __name__ == '__main__':
    unittest.main()
