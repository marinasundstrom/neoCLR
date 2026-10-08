"""Guard neoCLR's public and internal source-interface naming convention."""
from pathlib import Path
import re
import unittest

ROOT = Path(__file__).resolve().parents[1]


class InterfaceNamesTests(unittest.TestCase):
    def test_runtime_interfaces_do_not_use_dotnet_i_prefix(self):
        declarations = []
        for path in sorted((ROOT / 'runtime/raven/src').rglob('*.rvn')):
            for match in re.finditer(r'^\s*(?:(?:public|internal|private|sealed)\s+)*interface\s+(\w+)',
                                     path.read_text(), re.MULTILINE):
                declarations.append((str(path.relative_to(ROOT)), match.group(1)))
        self.assertTrue(declarations, 'No runtime interface declarations were audited')
        self.assertEqual([], [(path, name) for path, name in declarations
                              if re.match(r'I[A-Z]', name)])

    def test_runtime_neoil_interfaces_do_not_use_dotnet_i_prefix(self):
        declarations = []
        for path in sorted((ROOT / 'runtime').rglob('*.neoil')):
            for match in re.finditer(r'^\.interface\s+([\w.]+)', path.read_text(), re.MULTILINE):
                declarations.append((str(path.relative_to(ROOT)), match.group(1)))
        self.assertTrue(declarations, 'No neoIL interface declarations were audited')
        self.assertEqual([], [(path, name) for path, name in declarations
                              if re.match(r'I[A-Z]', name.rsplit('.', 1)[-1])])


if __name__ == '__main__':
    unittest.main()
