"""Build and run the Byte Copy sample and checks with a matching local toolchain."""
import argparse
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile


def run(command, **kwargs):
    result = subprocess.run(command, capture_output=True, text=True, timeout=120, **kwargs)
    if result.returncode:
        raise RuntimeError(result.stdout + result.stderr)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--toolchain-root', type=Path, required=True)
    args = parser.parse_args()
    toolchain = args.toolchain_root.resolve()
    here = Path(__file__).resolve().parent
    env = dict(os.environ, NeoCLRRoot=str(toolchain), RavenSdkRoot=str(toolchain / 'raven-sdk'))
    for source, expected in [
        ('Main.rvn', (here / 'expected.txt').read_text()),
        ('Checks.rvn', 'Extreme and empty ranges: passed\nPartial I/O, EOF and output failure: passed\nManaged owner retained through GC: passed\n'),
    ]:
        with tempfile.TemporaryDirectory(prefix='neoclr-byte-copy-') as folder:
            root = Path(folder)
            for name in ['ByteCopy.rvn', 'ByteCopy.rvnproj']:
                shutil.copyfile(here / name, root / name)
            shutil.copyfile(here / source, root / 'Main.rvn')
            build = run(['dotnet', 'msbuild', str(root / 'ByteCopy.rvnproj'), '-nologo', '-v:minimal'], env=env)
            if 'warning ' in build.stdout:
                raise RuntimeError('Unexpected sample warnings:\n' + build.stdout)
            result = run([str(toolchain / 'bin/neoclr'), 'run', str(root / 'bin/neoclr/Debug/App.neoil'), '--system', str(toolchain / 'lib/System.neoil'), '--gc-stats'])
            if result.stdout != expected:
                raise AssertionError(f'{source}: unexpected output\n{result.stdout}\n{result.stderr}')
            if source == 'Checks.rvn':
                collections = re.search(r'collections=(\d+)', result.stderr)
                if not collections or int(collections[1]) == 0:
                    raise AssertionError('GC check did not trigger collection: ' + result.stderr)
            print(f'{source}: passed', flush=True)
            if source == 'Checks.rvn':
                # Each fresh invocation respects the default instruction budget.
                for mode in range(2):
                    for offset in range(-1, 7):
                        matrix = run([str(toolchain / 'bin/neoclr'), 'run', str(root / 'bin/neoclr/Debug/App.neoil'), '--system', str(toolchain / 'lib/System.neoil'), '--', str(mode), str(offset)])
                        if matrix.stdout != 'Checked range/alias cases:\n64\n':
                            raise AssertionError(matrix.stdout + matrix.stderr)
                    print(f'Range matrix mode {mode}: 512 cases passed', flush=True)



if __name__ == '__main__':
    main()
