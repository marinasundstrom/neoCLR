#!/usr/bin/env python3
"""Compare current heap/value async policies; retain logs without treating rejection as support."""
import argparse
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--toolchain-root', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
bundle = args.toolchain_root.resolve()
output = args.output.resolve()
output.mkdir(parents=True, exist_ok=True)
source = Path(__file__).with_name('Main.rvn')
env = dict(os.environ, NeoCLRRoot=str(bundle), RavenSdkRoot=str(bundle / 'raven-sdk'))
for name, heap in [('heap', 'true'), ('value', 'false')]:
    with tempfile.TemporaryDirectory(prefix='neoclr-value-async-') as folder:
        root = Path(folder)
        shutil.copyfile(source, root / 'Main.rvn')
        (root / 'Probe.rvnproj').write_text(f'''<Project DefaultTargets="Build">
  <Import Project="$(NeoCLRRoot)/build/NeoCLR.Raven.props" />
  <PropertyGroup><RavenHeapAsyncStateMachines>{heap}</RavenHeapAsyncStateMachines></PropertyGroup>
  <ItemGroup><Compile Include="Main.rvn" /></ItemGroup>
  <Import Project="$(NeoCLRRoot)/build/NeoCLR.Raven.targets" />
</Project>
''')
        build = subprocess.run(['dotnet', 'msbuild', str(root / 'Probe.rvnproj'),
                                '-nologo', '-v:minimal', '-p:Configuration=Release'],
                               env=env, capture_output=True, text=True, timeout=120)
        (output / f'{name}-build.log').write_text(build.stdout + build.stderr)
        assemblies = list((root / 'obj').glob('**/compiled/Probe.dll'))
        if assemblies:
            shutil.copyfile(assemblies[-1], output / f'{name}.dll')
        if build.returncode:
            print(f'{name}: build/import blocked; see {output / (name + "-build.log")}')
            if heap == 'true':
                raise SystemExit('The existing heap baseline must build successfully')
            continue
        run = subprocess.run([str(bundle / 'bin/neoclr'), 'run',
                              str(root / 'bin/neoclr/Release/App.neoil'),
                              '--system', str(bundle / 'lib/System.neoil')],
                             capture_output=True, text=True, timeout=120)
        (output / f'{name}-run.log').write_text(run.stdout + run.stderr)
        if run.returncode or run.stdout != 'Pending\n42\n' or run.stderr:
            raise SystemExit(f'{name}: runtime/output check failed')
        print(f'{name}: pending continuation returned 42')
