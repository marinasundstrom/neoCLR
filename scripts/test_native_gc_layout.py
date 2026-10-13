"""Qualify collector source relocation using only the packaged support closure."""
import importlib.util
from pathlib import Path
import platform
import shutil
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('native_build_layout', ROOT / 'scripts/build-native-project.py')
builder = importlib.util.module_from_spec(spec)
spec.loader.exec_module(builder)


class NativeGcLayoutTests(unittest.TestCase):
    def test_relocated_package_and_instrumented_source(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for source in builder.support_files():
                destination = root / source.relative_to(ROOT)
                destination.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(source, destination)
            base = root / 'docs/experiments/aot-console'
            shutil.copy2(ROOT / 'docs/experiments/aot-console/native-gc-test.c', base)
            canonical = root / 'runtime/native/gc/native-gc.c'
            windows = platform.system() == 'Windows'
            if windows:
                cc = ['cl', '/nologo', '/W4', '/WX', '/O2', '/std:c11', '/experimental:c11atomics', '/MT', '/DNEOCLR_NATIVE_GC', '/I' + str(base)]
            else:
                cc = ['clang', '-std=c11', '-O2', '-Wall', '-Wextra', '-Werror', '-DNEOCLR_NATIVE_GC', '-I', str(base)]
            # Profilers copy and instrument the canonical text outside its directory.
            copied = root / 'profile-collector.c'
            copied.write_text(canonical.read_text().replace('    statistics.allocations++;',
                '    statistics.allocations += 1;'))
            for name, source in [('compatibility', base / 'native-gc.c'), ('canonical', canonical), ('profile', copied)]:
                binary = root / (name + ('.exe' if windows else ''))
                probe = base / 'root-probe.c' if name == 'compatibility' else root / 'runtime/native/gc/root-probe.c'
                command = cc + [str(source), str(probe), str(base / 'native-gc-test.c')]
                command += ['/Fe:' + str(binary)] if windows else ['-o', str(binary)]
                result = subprocess.run(command, cwd=root, capture_output=True, text=True, timeout=60)
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
                result = subprocess.run([str(binary)], capture_output=True, text=True, timeout=30)
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

            fixture = root / 'arena-context-test.c'
            shutil.copy2(ROOT / 'tools/native/arena-context-test.c', fixture)
            for gc in [False, True]:
                for direct in [False, True]:
                    with self.subTest(gc=gc, direct=direct):
                        arena = (root / 'runtime/native/memory' if direct else base) / 'text-arena.c'
                        command = [arg for arg in cc if gc or arg not in ('-DNEOCLR_NATIVE_GC', '/DNEOCLR_NATIVE_GC')]
                        command += [str(arena), str(fixture)]
                        if gc:
                            source_dir = root / 'runtime/native/gc' if direct else base
                            command += [str(source_dir / name) for name in ('native-gc.c', 'root-probe.c')]
                        binary = root / ('arena-' + str(gc) + '-' + str(direct) + ('.exe' if windows else ''))
                        command += ['/Fe:' + str(binary)] if windows else ['-o', str(binary)]
                        result = subprocess.run(command, cwd=root, capture_output=True, text=True, timeout=60)
                        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
                        result = subprocess.run([str(binary)], capture_output=True, text=True, timeout=30)
                        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

            fixture = root / 'fault-render-test.c'
            shutil.copy2(ROOT / 'tools/native/fault-render-test.c', fixture)
            for direct in [False, True]:
                fault_dir = root / ('runtime/native/faults' if direct else 'docs/experiments/aot-fault-details')
                binary = root / ('fault-' + str(direct) + ('.exe' if windows else ''))
                includes = ['/I' + str(fault_dir)] if windows else ['-I', str(fault_dir)]
                command = cc + includes + [str(fault_dir / 'render.c'), str(fixture)]
                command += ['/Fe:' + str(binary)] if windows else ['-o', str(binary)]
                result = subprocess.run(command, cwd=root, capture_output=True, text=True, timeout=60)
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
                result = subprocess.run([str(binary)], capture_output=True, text=True, timeout=30)
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

            spec = importlib.util.spec_from_file_location('gc_phases', ROOT / 'benchmarks/http-scheduling/gc_phases.py')
            phases = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(phases)
            command, _ = phases.instrument(root, root, [str(base / 'native-gc.c'), str(base / 'root-probe.c')])
            for source in command[:2]:
                output = root / (Path(source).stem + ('.obj' if windows else '.o'))
                flags = ['/c', '/Fo' + str(output)] if windows else ['-c', '-o', str(output)]
                result = subprocess.run(cc + flags + [source], cwd=root, capture_output=True, text=True, timeout=60)
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
