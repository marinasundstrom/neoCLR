import importlib.util
from pathlib import Path
import unittest
spec = importlib.util.spec_from_file_location('native_build', Path(__file__).with_name('build-native-project.py'))
build = importlib.util.module_from_spec(spec)
spec.loader.exec_module(build)

class HttpWorkerBuildTests(unittest.TestCase):
    def test_supported_explicit_counts(self):
        for count in range(1, 5):
            build.validate_http_workers(count, build.PROFILES['http'], None, False, False)

    def test_reject_unsupported_hosts_and_modes(self):
        for profile in build.PROFILES.values():
            if profile != build.PROFILES['http']:
                with self.assertRaises(ValueError):
                    build.validate_http_workers(2, profile, None, False, False)
        for bootstrap, tls, packaged in [('Start', False, False), (None, True, False), (None, False, True)]:
            with self.assertRaises(ValueError):
                build.validate_http_workers(2, build.PROFILES['http'], bootstrap, tls, packaged)
        for count in [-1, 5]:
            with self.assertRaises(ValueError):
                build.validate_http_workers(count, build.PROFILES['http'], None, False, False)

    def test_default_does_not_restrict_existing_profiles(self):
        for profile in build.PROFILES.values():
            build.validate_http_workers(0, profile, 'Start', True, True)

if __name__ == '__main__':
    unittest.main()
