"""Package the runtime API POC with a published bridge and no checkout-path dependency.

Build the separate Raven SDK/VSIX first using RELEASING.md. This does not publish.
"""
import argparse
from datetime import datetime, timezone
import hashlib
import json
import platform
import sys
from pathlib import Path
import shutil
import subprocess
from collection_library import build, ROOT

HERE = Path(__file__).resolve().parent
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--raven', required=True, type=Path)
parser.add_argument('--sdk', required=True, type=Path)
parser.add_argument('--runtime', required=True, type=Path)
parser.add_argument('--output', required=True, type=Path)
parser.add_argument('--version', required=True)
args = parser.parse_args()
if sys.platform != 'darwin' or platform.machine() != 'arm64':
    parser.error('This packaging slice supports the tested macOS arm64 host only.')

def git(path, *arguments):
    return subprocess.check_output(['git', '-C', str(path), *arguments], text=True).strip()

raven, sdk, runtime, output = (p.resolve() for p in (args.raven, args.sdk, args.runtime, args.output))
for repository in (ROOT, raven):
    if git(repository, 'status', '--porcelain'):
        raise SystemExit('Commit/review source changes before packaging: ' + str(repository))
if not runtime.is_file() or not (sdk / 'tools/language-server/Raven.LanguageServer.dll').is_file():
    raise SystemExit('Build neoCLR and the experimental Raven SDK first.')
output.mkdir(parents=True, exist_ok=False)
for directory in ('bin', 'lib', 'demo', 'msbuild-demo', 'tools', 'docs/experiments/raven-target'):
    (output / directory).mkdir(parents=True, exist_ok=True)
subprocess.run(['dotnet', 'publish', str(HERE / 'Probe.csproj'), '-c', 'Release',
    '-p:RavenRoot=' + str(raven), '-p:BuildProjectReferences=false', '-p:WarningLevel=0',
    '-o', str(output / 'tools/bridge')], check=True)
probe = output / 'tools/bridge/Probe.dll'
probe_output = output / 'metadata-build'
subprocess.run(['dotnet', str(probe), '--interfaces', str(probe_output)], check=True)
shutil.copyfile(probe_output / 'NeoCLR.CoreProbe.dll', output / 'demo/NeoCLR.CoreProbe.dll')
shutil.copy2(runtime, output / 'bin/neoclr')
(output / 'lib/System.neoil').write_text(build(ROOT / 'runtime/System.neoil'))
shutil.copytree(sdk / 'tools/language-server', output / 'tools/server')
for name in ('run_project.py', 'runner_options.py', 'configure_tasks.py', 'verify_project.py',
             'verify_editor.py', 'verify_msbuild.py', 'verify_array_api.py', 'verify_neoil.py', 'verify_file_project.py', 'verify_process.py', 'verify_clock.py', 'verify_error_values.py',
             'verify_unions.py', 'verify_matches.py', 'verify_delegates.py', 'verify_native_buffer.py',
             'verify_application.py', 'verify_orders.py', 'verify_queries.py', 'verify_collection_capabilities.py', 'verify_compiler_target.py', 'verify_library_import.py'):
    shutil.copyfile(HERE / name, output / 'tools' / name)
shutil.copytree(ROOT / 'examples/preview', output / 'samples/neoil')
shutil.copytree(HERE / 'samples', output / 'tools/samples')
shutil.copytree(HERE / 'samples', output / 'docs/experiments/raven-target/samples')
for path in (ROOT / 'docs').glob('*.md'):
    shutil.copyfile(path, output / 'docs' / path.name)
for name in ('runtime-api-inventory.json', 'runtime-api-coverage.json', 'RELEASING.md', 'VSCODE.md', 'README.md'):
    shutil.copyfile(HERE / name, output / 'docs/experiments/raven-target' / name)
for name in ('README.md', 'configure.py'):
    shutil.copyfile(HERE / 'bundle' / name, output / name)
shutil.copyfile(HERE / 'bundle/Demo.rvnproj', output / 'demo/Demo.rvnproj')
shutil.copyfile(HERE / 'samples/library-propagation-workflow.rvn', output / 'demo/Main.rvn')
shutil.copytree(ROOT / 'build', output / 'build')
shutil.copyfile(HERE / 'msbuild/Demo.rvnproj', output / 'msbuild-demo/Demo.rvnproj')
shutil.copyfile(HERE / 'samples/library-propagation-workflow.rvn', output / 'msbuild-demo/Main.rvn')
# Preserve upstream attribution alongside every distributed implementation.
for name in ('LICENSE', 'THIRD_PARTY_NOTICES.md'):
    shutil.copyfile(ROOT / name, output / name)
shutil.copytree(ROOT / 'third-party', output / 'third-party')
(output / 'licenses/Raven').mkdir(parents=True)
for name in ('LICENSE', 'THIRD-PARTY-NOTICES.txt'):
    shutil.copyfile(raven / name, output / 'licenses/Raven' / name)

# Probe construction artifacts are not part of the runnable distribution.
shutil.rmtree(probe_output)
manifest = {'version': args.version, 'createdUtc': datetime.now(timezone.utc).isoformat(),
    'distribution': 'local experiment; not published', 'platform': 'osx-arm64',
    'neoCLRRevision': git(ROOT, 'rev-parse', 'HEAD'), 'ravenRevision': git(raven, 'rev-parse', 'HEAD'),
    'ravenBranch': git(raven, 'branch', '--show-current'), 'sdkVersion': (sdk / 'VERSION').read_text().strip(),
    'dotnetSdk': subprocess.check_output(['dotnet', '--version'], text=True).strip(),
    'validation': 'Not yet validated; see separate validation record after package checks.',
    'files': {str(p.relative_to(output)): hashlib.sha256(p.read_bytes()).hexdigest()
              for p in sorted(output.rglob('*')) if p.is_file()}}
(output / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
print(output)
