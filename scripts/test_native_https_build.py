import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location('native_build', Path(__file__).with_name('build-native-project.py'))
build = importlib.util.module_from_spec(spec)
spec.loader.exec_module(build)


class HttpsBuildPolicyTests(unittest.TestCase):
    def test_explicit_source_http_capability(self):
        build.validate_https_server(True, build.PROFILES['http'], None, False)
        build.validate_https_server(True, build.PROFILES['windows-http'], None, False)
        for profile in ('console', 'windows-console'):
            with self.subTest(profile=profile), self.assertRaisesRegex(ValueError, 'HTTP profile'):
                build.validate_https_server(True, build.PROFILES[profile], None, False)
        with self.assertRaisesRegex(ValueError, 'bootstrap'):
            build.validate_https_server(True, build.PROFILES['http'], 'Start', False)
        with self.assertRaisesRegex(ValueError, 'packaged'):
            build.validate_https_server(True, build.PROFILES['http'], None, True)

    def test_tls_archive_matches_host_target(self):
        for windows, target, archive in [(False, 'aarch64-apple-darwin', 'libneoclr_native_tls.a'),
                                          (True, 'x86_64-pc-windows-msvc', 'neoclr_native_tls.lib')]:
            command, path = build.native_tls_build(windows)
            self.assertEqual(command[command.index('--target') + 1], target)
            self.assertEqual(path.name, archive)
            self.assertEqual('--config' in command, windows)

    def test_ordinary_builds_do_not_gain_tls_requirements(self):
        for profile in build.PROFILES.values():
            build.validate_https_server(False, profile, 'ExistingBootstrap', True)


if __name__ == '__main__':
    unittest.main()
