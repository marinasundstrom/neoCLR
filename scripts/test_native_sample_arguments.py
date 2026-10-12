"""Installed sample arguments remain guest arguments, not runner options."""
import importlib.util
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

from native_bundle_samples import configure_editor_tasks

spec = importlib.util.spec_from_file_location('sample_runner', Path(__file__).with_name('run-native-sample.py'))
runner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runner)


class SampleArguments(unittest.TestCase):
    def test_guest_arguments_preserve_order_and_text(self):
        with tempfile.TemporaryDirectory(prefix='sample arguments ') as directory:
            root = Path(directory)
            (root / 'samples/client').mkdir(parents=True)
            (root / 'manifest.json').write_text(json.dumps(dict(
                samples={'client': {}}, runtime='bin/neoclr', runtimeSeed='lib/System.neox', modules=[])))
            guest = ['http://127.0.0.1:1234/', 'Café team', '--bundle', 'guest value']
            with patch.object(sys, 'argv', ['runner', 'client', '--bundle', str(root), '--', *guest]), \
                    patch.object(runner.subprocess, 'run') as compile_project, \
                    patch.object(runner.subprocess, 'call', return_value=0) as execute:
                with self.assertRaises(SystemExit) as exit_result:
                    runner.main()
            self.assertEqual(exit_result.exception.code, 0)
            compile_project.assert_called_once()
            command = execute.call_args.args[0]
            self.assertEqual(command[command.index('--') + 1:], guest)
            self.assertEqual(command[0], str(root.resolve() / 'bin/neoclr'))

    def test_http_tasks_prompt_for_required_program_inputs(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            names = ['http-json-server', 'http-json-client', 'json']
            for name in names:
                (root / 'samples' / name).mkdir(parents=True)
            configure_editor_tasks(root, names)
            for name, identifier in [('http-json-server', 'requestCount'), ('http-json-client', 'serverUrl')]:
                tasks = json.loads((root / 'samples' / name / '.vscode/tasks.json').read_text())
                self.assertEqual(tasks['tasks'][0]['args'][-2:], ['--', '${input:' + identifier + '}'])
                self.assertEqual(tasks['inputs'][0]['id'], identifier)
            tasks = json.loads((root / 'samples/json/.vscode/tasks.json').read_text())
            self.assertNotIn('inputs', tasks)
            self.assertEqual(tasks['tasks'][0]['args'][-1], 'json')


if __name__ == '__main__':
    unittest.main()
