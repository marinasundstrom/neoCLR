import unittest
from unittest.mock import patch
import host


class ReleaseHosts(unittest.TestCase):
    def test_release_profiles_and_executable_names(self):
        self.assertEqual(host.target('Darwin', 'arm64'),
                         dict(console='console', http='http', executable='app'))
        for arch in ('AMD64', 'x86_64'):
            self.assertEqual(host.target('Windows', arch),
                             dict(console='windows-console', http='windows-http', executable='app.exe'))

    def test_unqualified_hosts_are_rejected(self):
        for system, arch in [('Linux', 'x86_64'), ('Windows', 'arm64'), ('Darwin', 'x86_64')]:
            with self.assertRaises(ValueError):
                host.target(system, arch)

    def test_missing_load_average_is_not_reported_as_idle(self):
        with patch('host.os', spec=['environ']):
            self.assertIsNone(host.load_average())

    def test_windows_process_snapshot_does_not_use_posix_tools(self):
        with patch('host.platform.system', return_value='Windows'), \
             patch('host.subprocess.check_output', return_value='processes') as run:
            result = host.process_snapshot()
        self.assertEqual(result['command'], ['tasklist', '/FO', 'CSV'])
        self.assertEqual(run.call_args.args[0], result['command'])


if __name__ == '__main__':
    unittest.main()
