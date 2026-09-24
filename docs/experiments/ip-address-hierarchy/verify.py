"""Run the isolated address hierarchy on the supplied matching target bundle."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--toolchain-root', type=Path, required=True)
parser.add_argument('--runner', type=Path, required=True)
parser.add_argument('--public', action='store_true', help='Use the public API from the matching bundle')
args = parser.parse_args()
bundle = args.toolchain_root.resolve()
here = Path(__file__).resolve().parent
with tempfile.TemporaryDirectory(prefix='neoclr-address-hierarchy-') as folder:
    root = Path(folder)
    for name in ('Main.rvn', 'Address.rvn', 'AddressHierarchy.rvnproj'):
        shutil.copyfile(here / name, root / name)
    if args.public:
        shutil.copyfile(here / 'Public.rvn', root / 'Main.rvn')
        project = root / 'AddressHierarchy.rvnproj'
        project.write_text(project.read_text().replace('<Compile Include="Address.rvn" />', ''))
    cases = json.loads((here / 'cases.json').read_text())
    # Raven string literals use the JSON escapes in these ASCII/UTF-8 fixtures.
    statements = [f'    Valid({json.dumps(text, ensure_ascii=False)}, {json.dumps(expected)})'
                  for text, expected in cases['valid'].items()]
    statements += [f'    Invalid({json.dumps(text, ensure_ascii=False)})' for text in cases['invalid']]
    main = root / 'Main.rvn'
    main.write_text(main.read_text().replace('    // PARSER_CASES', '\n'.join(statements)))
    env = dict(os.environ, NeoCLRRoot=str(bundle), RavenSdkRoot=str(bundle / 'raven-sdk'))
    build = subprocess.run(['dotnet', 'msbuild', str(root / 'AddressHierarchy.rvnproj'),
                            '-nologo', '-v:minimal'], env=env, capture_output=True,
                           text=True, timeout=150)
    assert build.returncode == 0, build.stdout + build.stderr
    run = subprocess.run([str(args.runner.resolve()), str(root / 'bin/neoclr/Debug/App.neoil'),
                          str(bundle / 'lib/System.neoil'), '256', '100000000'],
                         capture_output=True, text=True, timeout=300)
    assert run.returncode == 0, run.stdout + run.stderr
    assert 'Closed address hierarchy value and GC checks passed' in run.stdout, run.stdout
    assert 'live=0' in run.stderr, run.stderr
    print(run.stdout + run.stderr)

    # The family is closed across source files, not merely marked abstract.
    (root / 'External.rvn').write_text(('import System.Networking.*\n' if args.public else '') + 'class ExternalAddress : IPAddress {\n'
                                     '    init(bytes: byte[]) : base(bytes) { }\n}\n')
    project = root / 'AddressHierarchy.rvnproj'
    project.write_text(project.read_text().replace('</ItemGroup>',
                       '<Compile Include="External.rvn" /></ItemGroup>'))
    rejected = subprocess.run(['dotnet', 'msbuild', str(project), '-nologo', '-v:minimal'],
                              env=env, capture_output=True, text=True, timeout=150)
    diagnostic = rejected.stdout + rejected.stderr
    assert rejected.returncode != 0 and ('RAV0306' if args.public else 'RAV0334') in diagnostic and 'IPAddress' in diagnostic, diagnostic
    print('External address branch rejected by Raven')

    comparison = root / 'reference'
    comparison.mkdir()
    shutil.copyfile(here / 'Reference.cs', comparison / 'Program.cs')
    (comparison / 'Reference.csproj').write_text(
        '<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><TargetFramework>net10.0</TargetFramework>'
        '<OutputType>Exe</OutputType><ImplicitUsings>enable</ImplicitUsings></PropertyGroup></Project>')
    baseline = subprocess.run(['dotnet', 'run', '--project', str(comparison / 'Reference.csproj'),
                               '--', str(here / 'cases.json')],
                              capture_output=True, text=True, timeout=150)
    assert baseline.returncode == 0, baseline.stdout + baseline.stderr
    print(baseline.stdout)
