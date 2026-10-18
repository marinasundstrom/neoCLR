import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location('load_analysis', Path(__file__).with_name('analyze.py'))
analysis = importlib.util.module_from_spec(spec)
spec.loader.exec_module(analysis)


class AnalysisContract(unittest.TestCase):
    def test_losses_and_memory_windows_remain_visible(self):
        m = dict(durationSeconds=20,
                 requests=[dict(index=0, intendedSeconds=0, endSeconds=.1, result='ok',
                                offeredLatencySeconds=.1, dispatchLagSeconds=.01),
                           dict(index=1, intendedSeconds=11, result='generator_inflight_limit',
                                dispatchLagSeconds=.5)],
                 resources=[dict(atSeconds=1, rssKiB=128), dict(atSeconds=9, rssKiB=130),
                            dict(atSeconds=11, rssKiB=130), dict(atSeconds=19, rssKiB=132),
                            dict(atSeconds=21, rssKiB=200)],
                 outcomes={'ok': 1, 'generator_inflight_limit': 1},
                 successfulCompletionsPerSecond=.05, meetsProvisionalTarget=False,
                 offeredLatencySeconds={'p99': .1}, dispatchLagSeconds={'p99': .5},
                 serverCpuSecondsPerCompletion=.02, sampledPeakRssKiB=200)
        trial = analysis.summarize(dict(completed=True, trials=[dict(
            round=0, rate=.1, concurrency=1, measurement=m)]))['trials'][0]
        self.assertFalse(trial['target'])
        self.assertEqual(trial['seconds'][11]['outcomes'], {'generator_inflight_limit': 1})
        self.assertEqual(sum(s['completions'] for s in trial['seconds']), 1)
        self.assertEqual(trial['rssFirstTenSecondsMedianKiB'], 129)
        self.assertEqual(trial['rssLastTenSecondsMedianKiB'], 131)
        self.assertEqual(trial['rssPeakKiB'], 200)
        self.assertEqual(trial['phaseLatencies']['connect']['samples'], 0)
        self.assertEqual(len(trial['slowest']), 1)

    def test_incomplete_trials_are_not_dropped(self):
        result = analysis.summarize(dict(completed=False, trials=[dict(rate=100, error='failed')]))
        self.assertFalse(result['completed'])
        self.assertEqual(result['trials'], [dict(rate=100, error='failed')])


if __name__ == '__main__':
    unittest.main()
