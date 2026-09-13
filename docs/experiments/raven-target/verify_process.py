"""Run process APIs against controlled host arguments, environment and byte input."""
import argparse
from runner_options import add_toolchain_arguments, runner_arguments
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('project', type=Path)
add_toolchain_arguments(parser)
parser.add_argument('--runtime', type=Path, required=True)
args = parser.parse_args()
bridge = Path(__file__).resolve().parent
with tempfile.TemporaryDirectory(prefix='neoclr-process-') as temporary:
    root = Path(temporary)
    for name in ('Demo.rvnproj', 'NeoCLR.CoreProbe.dll'):
        shutil.copyfile(args.project.resolve().parent / name, root / name)
    build = [sys.executable, str(bridge / 'run_project.py'), str(root / 'Demo.rvnproj'),
             *runner_arguments(args), '--runtime', str(args.runtime.resolve()), '--build-only']
    for name in ('environment', 'console'):
        before = set(root.rglob('App.neoil'))
        (root / 'Main.rvn').write_text((bridge / f'samples/library-{name}.rvn').read_text())
        compiled = subprocess.run(build, capture_output=True, text=True, timeout=90)
        if compiled.returncode:
            raise AssertionError(compiled.stdout + compiled.stderr)
        created = set(root.rglob('App.neoil')) - before
        if len(created) != 1:
            raise AssertionError('Expected one fresh executable')
        artifact = created.pop()
        command = [str(args.runtime.resolve()), 'run', str(artifact), '--system', str(artifact.parent / 'System.Collections.neoil')]
        environment = os.environ.copy()
        environment.update(NEOCLR_API_PRESENT='Value', NEOCLR_API_EMPTY='')
        environment.pop('NEOCLR_API_MISSING', None)
        if name == 'environment':
            command += ['--', 'original']
            expected = f'2\noriginal\nChanged\noriginal\n{root.resolve()}\nValue\n\nMissing\nEnvironmentUnavailable\n'
        else:
            expected = '0\n255\nEnd of input\n'
        run = subprocess.run(command, input=b'\x00\xff', capture_output=True, cwd=root, env=environment, timeout=30)
        if run.returncode or run.stdout.decode() != expected:
            raise AssertionError(repr(run.stdout) + repr(run.stderr))
        print(name + ': ' + repr(run.stdout.decode()))
