"""Compile saved Raven project sources, import, verify, and optionally run on neoCLR."""
import argparse
import json
from pathlib import Path
import subprocess
import tempfile
import shutil
from runner_options import add_toolchain_arguments

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('project', type=Path)
add_toolchain_arguments(parser)
parser.add_argument('--runtime', required=True, type=Path)
parser.add_argument('--build-only', action='store_true')
args = parser.parse_args()
project, runtime = args.project.resolve(), args.runtime.resolve()
if not project.is_file() or not runtime.is_file():
    parser.error('Project and built neoCLR executable must exist.')
bridge = Path(__file__).resolve().parent
builds = project.parent / '.neoclr-build'
builds.mkdir(exist_ok=True)
# Each attempt owns a new directory. A failed compilation can never run old output.
output = Path(tempfile.mkdtemp(prefix='build-', dir=builds)) / 'output'
try:
    if args.bridge:
        if not args.bridge.is_file():
            parser.error('Published bridge does not exist.')
        compiler = ['dotnet', str(args.bridge.resolve())]
    else:
        compiler = ['dotnet', 'run', '--project', str(bridge / 'Probe.csproj'),
            '-p:RavenRoot='+str(args.raven.resolve()), '-p:BuildProjectReferences=false', '-p:WarningLevel=0', '--']
    subprocess.run([*compiler, '--project', str(project), str(output)], cwd=bridge, check=True)
    artifact = output / 'App.neoil'
    mapping = json.loads(Path(str(artifact) + '.map.json').read_text())
    profile = mapping.get('RequiredLibraryProfile')
    system_arguments = []
    if profile == 'raven-collections':
        system = output / 'System.Collections.neoil'
        if args.system:
            shutil.copyfile(args.system.resolve(), system)
        elif args.bridge:
            parser.error('A published bridge requires --system for the collection target profile.')
        else:
            from collection_library import build, ROOT
            system.write_text(build(ROOT / 'runtime/System.neoil'))
        system_arguments = ['--system', str(system)]
    elif profile != 'bundled-system':
        raise SystemExit('Unsupported imported library profile: ' + str(profile))
    subprocess.run([str(runtime), 'verify', str(artifact), *system_arguments], check=True)
    print('Verified saved project: '+str(artifact), flush=True)
    if not args.build_only:
        subprocess.run([str(runtime), 'run', str(artifact), *system_arguments], check=True)
except subprocess.CalledProcessError as error:
    raise SystemExit(error.returncode)
