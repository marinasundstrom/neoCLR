import unittest
from jit_host_policy import ownership_command


class HostPolicyTests(unittest.TestCase):
    def test_default_removes_inherited_ownership(self):
        self.assertEqual(ownership_command(['clang', '-DNEOCLR_GC_OWNED_HEAP', '/DNEOCLR_GC_OWNED_HEAP=0', '-O2'], False), ['clang', '-O2'])

    def test_explicit_option_adds_one_define(self):
        self.assertEqual(ownership_command(['clang', '-DNEOCLR_GC_OWNED_HEAP=1'], True), ['clang', '-DNEOCLR_GC_OWNED_HEAP'])

    def test_split_definitions_and_other_flags(self):
        command = ['clang', '-D', 'NEOCLR_GC_OWNED_HEAP=0', '/D', 'NEOCLR_GC_OWNED_HEAP', '-DNEOCLR_GC_PACED', '-DOTHER=2']
        self.assertEqual(ownership_command(command, False), ['clang', '-DNEOCLR_GC_PACED', '-DOTHER=2'])

    def test_explicit_option_overrides_recipe_undefines(self):
        command = ['clang', '-UNEOCLR_GC_OWNED_HEAP', '/U', 'NEOCLR_GC_OWNED_HEAP', '-UOTHER']
        self.assertEqual(ownership_command(command, True), ['clang', '-DNEOCLR_GC_OWNED_HEAP', '-UOTHER'])

    def test_unqualified_platform_is_rejected(self):
        with self.assertRaises(ValueError):
            ownership_command(['cl'], True, windows=True)
        self.assertEqual(ownership_command(['cl', '/DNEOCLR_GC_OWNED_HEAP'], False, windows=True), ['cl'])


if __name__ == '__main__':
    unittest.main()
