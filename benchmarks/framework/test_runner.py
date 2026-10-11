import contextlib
import io
import json
from pathlib import Path
import sys
import tempfile
import unittest

from runner import run, sample, sha

GOOD = "import time; print('NEOBENCH/1 READY', flush=True); print('NEOBENCH/1 BEGIN', flush=True); time.sleep(.02); print('NEOBENCH/1 END'); print('2')"


class RunnerTests(unittest.TestCase):
    def test_valid_batch_accounts_for_operations(self):
        result = sample([sys.executable, '-c', GOOD], 2)
        self.assertEqual(result['operations'], 2)
        self.assertGreater(result['batchSeconds'], 0)
        self.assertAlmostEqual(result['operationsPerSecond'] * result['batchSeconds'], 2)

    def test_rejects_protocol_failures(self):
        for source in [GOOD.replace("print('2')", "print('3')"),
                       GOOD + "; print('extra')", GOOD + '; raise SystemExit(1)',
                       GOOD + "; import sys; sys.stderr.write('failure')",
                       GOOD.replace('NEOBENCH/1 BEGIN', 'NEOBENCH/1 END')]:
            with self.subTest(source=source), self.assertRaises(RuntimeError):
                sample([sys.executable, '-c', source], 2)

    def test_timeout_terminates_worker(self):
        with self.assertRaises(TimeoutError):
            sample([sys.executable, '-c', 'import time; time.sleep(30)'], 2, timeout=.1)

    def test_manifest_filter_accounting_and_input_changes(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            worker = root/'worker.py'
            worker.write_text(GOOD)
            manifest = root/'manifest.json'
            manifest.write_text(json.dumps(dict(schemaVersion=1,
                commands={m:[sys.executable,str(worker)] for m in ['interpreter','jit','aot']},
                inputs={str(worker):sha(worker)}, cases=['one','two'])))
            output = root/'result.json'
            with contextlib.redirect_stdout(io.StringIO()):
                result = run(manifest, output, 3, 0, 2, 'two')
            self.assertTrue(result['passed'])
            self.assertEqual(len(result['samples']), 9)
            self.assertEqual(set(x['case'] for x in result['samples']), {'two'})
            self.assertEqual(result['samples'][3]['mode'], 'jit')
            with self.assertRaises(ValueError):
                run(manifest, root/'unknown.json', 3, 0, 2, 'missing')
            with self.assertRaises(ValueError):
                run(manifest, output, 3, 0, 2)
            worker.write_text(GOOD+'\n# changed')
            with self.assertRaisesRegex(ValueError, 'input changed'):
                run(manifest, root/'changed.json', 3, 0, 2)


if __name__ == '__main__':
    unittest.main()
