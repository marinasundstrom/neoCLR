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

    def test_reflection_preparation_can_repeat_without_overwriting_a_listing(self):
        with tempfile.TemporaryDirectory() as directory:
            assembly = Path(directory) / 'App.dll'
            existing = assembly.with_suffix('.neoil')
            existing.write_text('Keep this caller-owned listing')
            outputs = []
            metadata = dict(name='Sample', revision=1, types=[dict(name='Native.T__506572736F6E')])
            def disassemble(command):
                listing = command[-1]
                self.assertFalse(listing.exists())
                outputs.append(listing)
                listing.write_text(json.dumps(metadata))
            for _ in range(2):
                args = runner.reflection_roots('neoclr', assembly, ['Person'], disassemble)
                roots = json.loads(args[1].read_text())
                self.assertEqual(roots['types'][0]['definition']['index'], 0)
            self.assertNotEqual(outputs[0], outputs[1])
            self.assertTrue(all(not path.exists() for path in outputs))
            self.assertEqual(existing.read_text(), 'Keep this caller-owned listing')

    def test_constructor_retention_resolves_exact_names_from_each_final_image(self):
        with tempfile.TemporaryDirectory() as directory:
            assembly = Path(directory) / 'App.dll'
            for ordinal in (0, 1):
                declarations = [dict(name='OpaqueA', origin=dict(name='App.Service')),
                                dict(name='OpaqueB', origin=dict(name='Other.Service'))]
                if ordinal:
                    declarations.reverse()
                def disassemble(command):
                    command[-1].write_text(json.dumps(dict(name='Sample', revision=1, types=declarations)))
                args = runner.reflection_roots('neoclr', assembly, ['App.Service'], disassemble, constructors=True)
                policy = json.loads(args[1].read_text())
                self.assertEqual(policy['schemaVersion'], 7)
                root = policy['types'][0]
                self.assertEqual(root['definition']['index'], ordinal)
                self.assertTrue(root['constructors'])
                self.assertTrue(root['construct'])
                self.assertFalse(root['properties'])
                self.assertFalse(root['fieldGetters'])
                with self.assertRaises(ValueError):
                    runner.reflection_roots('neoclr', assembly, ['Missing.Service'], disassemble, constructors=True)

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
