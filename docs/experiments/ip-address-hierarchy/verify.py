"""Run the isolated address hierarchy on the supplied matching target bundle."""
import argparse
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--toolchain-root', type=Path, required=True)
parser.add_argument('--runner', type=Path, required=True)
args = parser.parse_args()
bundle = args.toolchain_root.resolve()
here = Path(__file__).resolve().parent
with tempfile.TemporaryDirectory(prefix='neoclr-address-hierarchy-') as folder:
    root = Path(folder)
    for name in ('Main.rvn', 'AddressHierarchy.rvnproj'):
        shutil.copyfile(here / name, root / name)
    env = dict(os.environ, NeoCLRRoot=str(bundle), RavenSdkRoot=str(bundle / 'raven-sdk'))
    build = subprocess.run(['dotnet', 'msbuild', str(root / 'AddressHierarchy.rvnproj'),
                            '-nologo', '-v:minimal'], env=env, capture_output=True,
                           text=True, timeout=150)
    assert build.returncode == 0, build.stdout + build.stderr
    run = subprocess.run([str(args.runner.resolve()), str(root / 'bin/neoclr/Debug/App.neoil'),
                          str(bundle / 'lib/System.neoil'), '256', '10000000'],
                         capture_output=True, text=True, timeout=300)
    assert run.returncode == 0, run.stdout + run.stderr
    assert 'Closed address hierarchy value and GC checks passed' in run.stdout, run.stdout
    assert 'live=0' in run.stderr, run.stderr
    print(run.stdout + run.stderr)

    # The family is closed across source files, not merely marked abstract.
    (root / 'External.rvn').write_text('class ExternalAddress : IPAddress {\n'
                                     '    init(bytes: byte[]) : base(bytes) { }\n}\n')
    project = root / 'AddressHierarchy.rvnproj'
    project.write_text(project.read_text().replace('</ItemGroup>',
                       '<Compile Include="External.rvn" /></ItemGroup>'))
    rejected = subprocess.run(['dotnet', 'msbuild', str(project), '-nologo', '-v:minimal'],
                              env=env, capture_output=True, text=True, timeout=150)
    diagnostic = rejected.stdout + rejected.stderr
    assert rejected.returncode != 0 and 'RAV0334' in diagnostic and 'ExternalAddress' in diagnostic, diagnostic
    print('External address branch rejected by Raven')
