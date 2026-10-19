"""Focused native byte-window checks; no Raven/JIT execution is implied."""
from pathlib import Path
import importlib.util
import platform
import shutil
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]


class NativeMemoryViewTests(unittest.TestCase):
    def test_support_package_contains_window_header(self):
        spec = importlib.util.spec_from_file_location('native_builder', ROOT / 'scripts/build-native-project.py')
        builder = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(builder)
        self.assertIn(ROOT / 'runtime/native/memory/memory-views.h', builder.support_files())

    @unittest.skipUnless(shutil.which('clang') and platform.system() in ('Darwin', 'Linux'),
                         'requires POSIX clang; Windows uses validate-native-gc.py')
    def test_checked_windows_in_arena_and_gc(self):
        compiler = ['clang']
        sanitizer = 'address,undefined'
        if platform.system() == 'Darwin':
            clang = subprocess.check_output(['xcrun', '--sdk', 'macosx', '--find', 'clang'], text=True).strip()
            sdk = subprocess.check_output(['xcrun', '--sdk', 'macosx', '--show-sdk-path'], text=True).strip()
            compiler = [clang, '-isysroot', sdk]
            # The repository's macOS native harness uses UBSan/bounds.
            sanitizer = 'undefined,bounds'
        with tempfile.TemporaryDirectory() as directory:
            for gc, crypto in ((False, False), (True, False), (False, True), (True, True)):
                with self.subTest(gc=gc, crypto=crypto):
                    binary = Path(directory) / ('views-gc' if gc else 'views-arena')
                    memory = ROOT / 'runtime/native/memory'
                    sources = [(ROOT / 'docs/experiments/native-cryptography/slice-adapter-test.c' if crypto else memory / 'memory-views-test.c'), memory / 'text-arena.c']
                    flags = []
                    if gc:
                        flags = ['-DNEOCLR_NATIVE_GC']
                        sources += [ROOT / 'runtime/native/gc' / name
                                    for name in ('native-gc.c', 'root-probe.c')]
                    command = [*compiler, '-std=c11', '-Wall', '-Wextra', '-Werror',
                               '-fsanitize=' + sanitizer, *flags,
                               *map(str, sources), '-o', str(binary)]
                    result = subprocess.run(command, capture_output=True, text=True, timeout=60)
                    self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
                    result = subprocess.run([str(binary)], capture_output=True, text=True, timeout=60)
                    self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
