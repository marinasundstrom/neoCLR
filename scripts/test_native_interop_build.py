import importlib.util
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('native_build', Path(__file__).with_name('build-native-project.py'))
build = importlib.util.module_from_spec(spec)
spec.loader.exec_module(build)


class NativeInteropBuildTests(unittest.TestCase):
    def test_managed_options_reject_before_creating_output(self):
        with tempfile.TemporaryDirectory() as temporary:
            missing = Path(temporary) / 'missing'
            output = Path(temporary) / 'output'
            for profile in ('native-interop', 'native-library'):
                for option in ({'reflection_types': ['Object']}, {'bootstrap_root': 'Start'},
                               {'byte_array_limit': 1024}):
                    with self.subTest(profile=profile, option=option):
                        with self.assertRaisesRegex(ValueError, 'do not support'):
                            build.build(missing, missing, missing, output,
                                        profile=build.PROFILES[profile], **option)
                        self.assertFalse(output.exists())

    def test_failed_dependency_validation_removes_pending_library(self):
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary)
            pending = output / 'libapp.dylib.pending'
            def run(command, phase):
                if phase == 'native-compilation':
                    return '{"exports": []}'
                if phase == 'host-compile-link':
                    pending.write_bytes(b'linked object')
                    return ''
                return 'library:\n\t/unexpected/library.dylib (version 1)\n'
            with self.assertRaisesRegex(RuntimeError, 'Unexpected native dependencies'):
                build.compile_scalar_interop('aot', 'assembly', [], output,
                    build.PROFILES['native-library'], 'clang', 'sdk', run)
            self.assertFalse(pending.exists())
            self.assertFalse((output / 'libapp.dylib').exists())

    def test_kit_support_includes_interop_host_and_resolver(self):
        files = set(build.support_files())
        self.assertIn(build.ROOT / 'tools/native/interop-host.c', files)
        self.assertIn(build.ROOT / 'runtime/native/interop/imports.c', files)


if __name__ == '__main__':
    unittest.main()
