import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location('filewatch', Path(__file__).with_name('validate-filewatch.py'))
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class OwnershipCommandTests(unittest.TestCase):
    def test_windows_keeps_console_adapter_and_replaces_only_http_entry(self):
        original = ['cl', 'tools/windows-console-host.c', 'benchmarks/http-host.c', '/Fe:app.pending.exe']
        result = module.ownership_command(original, Path('ownership.c'), Path('ownership.exe'))
        self.assertEqual(result, ['cl', 'tools/windows-console-host.c', 'ownership.c', '/Fe:ownership.exe'])
        self.assertIn('tools/windows-console-host.c', original)

    def test_macos_replaces_http_entry_and_rejects_missing_or_duplicate_hosts(self):
        self.assertEqual(module.ownership_command(['cc', 'http-host.c', '-o', 'app'], Path('ownership.c'), Path('ownership')),
                         ['cc', 'ownership.c', '-o', 'ownership'])
        for command in (['cc', '-o', 'app'], ['cc', 'http-host.c', 'http-host.c', '-o', 'app']):
            with self.assertRaises(ValueError): module.ownership_command(command, Path('ownership.c'), Path('ownership'))
