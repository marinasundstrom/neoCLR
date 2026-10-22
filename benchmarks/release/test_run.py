import importlib.util
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('release_run', Path(__file__).with_name('run.py'))
runner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runner)


class ReleaseRunnerTests(unittest.TestCase):
    def exercise(self, outcome='pass'):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            bundle = root / 'bundle'
            bundle.mkdir()
            (bundle / 'manifest.json').write_text('{}')
            output = root / 'results'
            calls = []
            def execute(command, **kwargs):
                calls.append(command)
                result = output / ('first.json' if len(calls) == 1 else 'second.json')
                result.write_text(json.dumps({'passed': outcome != 'false'}))
                if outcome == 'drift':
                    (bundle / 'manifest.json').write_text('{"changed":true}')
                return subprocess.CompletedProcess(command, 1 if outcome == 'exit' else 0)
            plan = [('first', ['worker1'], output / 'first.json', True),
                    ('second', ['worker2'], output / 'second.json', True)]
            with patch.object(runner.platform, 'platform', return_value='test'), patch.object(runner, 'target', return_value={}), patch.object(runner, 'cpu_description', return_value='test'), patch.object(runner.subprocess, 'check_output', return_value='test'), patch.object(runner, 'steps', return_value=plan), patch.object(runner.subprocess, 'run', side_effect=execute):
                if outcome == 'pass':
                    runner.run(bundle, output, 5)
                else:
                    with self.assertRaises(RuntimeError):
                        runner.run(bundle, output, 5)
            return calls, json.loads((output / 'release-benchmarks.json').read_text())

    def test_success_preserves_order_and_evidence_hashes(self):
        calls, report = self.exercise()
        self.assertEqual(calls, [['worker1'], ['worker2']])
        self.assertTrue(report['passed'])
        self.assertTrue(all(len(row['resultSha256']) == 64 for row in report['steps']))

    def test_failed_process_or_report_stops_measurement(self):
        for outcome in ('exit', 'false'):
            with self.subTest(outcome=outcome):
                calls, report = self.exercise(outcome)
                self.assertEqual(len(calls), 1)
                self.assertFalse(report['passed'])
                self.assertIn('error', report)

    def test_manifest_drift_rejects_success(self):
        _, report = self.exercise('drift')
        self.assertFalse(report['passed'])
        self.assertIn('changed', report['error'])

    def test_plan_covers_execution_and_http(self):
        plan = runner.steps(Path('/bundle'), Path('/output'), 5)
        self.assertEqual([step[0] for step in plan], ['prepare', 'execution', 'cpu-native', 'http-plain', 'http-json'])


if __name__ == '__main__':
    unittest.main()
