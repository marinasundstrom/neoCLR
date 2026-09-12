"""Compile saved Raven project sources, import, verify, and optionally run on neoCLR."""
import argparse
from pathlib import Path
import subprocess
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('project', type=Path)
parser.add_argument('--raven', required=True, type=Path)
parser.add_argument('--runtime', required=True, type=Path)
parser.add_argument('--build-only', action='store_true')
args = parser.parse_args()
project, raven, runtime = args.project.resolve(), args.raven.resolve(), args.runtime.resolve()
if not project.is_file() or not runtime.is_file():
    parser.error('Project and built neoCLR executable must exist.')
bridge = Path(__file__).resolve().parent
builds = project.parent / '.neoclr-build'
builds.mkdir(exist_ok=True)
# Each attempt owns a new directory. A failed compilation can never run old output.
output = Path(tempfile.mkdtemp(prefix='build-', dir=builds)) / 'output'
try:
    subprocess.run(['dotnet', 'run', '--project', str(bridge / 'Probe.csproj'),
        '-p:RavenRoot='+str(raven), '-p:BuildProjectReferences=false', '-p:WarningLevel=0',
        '--', '--project', str(project), str(output)], cwd=bridge, check=True)
    artifact = output / 'App.neoil'
    subprocess.run([str(runtime), 'verify', str(artifact)], check=True)
    print('Verified saved project: '+str(artifact), flush=True)
    if not args.build_only:
        subprocess.run([str(runtime), 'run', str(artifact)], check=True)
except subprocess.CalledProcessError as error:
    raise SystemExit(error.returncode)
