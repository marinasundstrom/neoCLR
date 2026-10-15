"""Verify integrated native project profiles using disposable project copies."""
import argparse
from pathlib import Path
import shutil
import subprocess
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--compiler', type=Path, required=True)
args = parser.parse_args()
source = Path(__file__).resolve().parent
root = source.parents[3]
props = root / 'target/tuple-names-libraries-v2/NeoCLR.ClassLibrary.props'
assert props.is_file(), f'Missing matching library bundle: {props}'
with tempfile.TemporaryDirectory(prefix='structural-project-') as temporary:
    project = Path(temporary) / 'project'
    shutil.copytree(source, project, ignore=shutil.ignore_patterns('bin', 'obj', '__pycache__'))
    for file in project.rglob('*.rvnproj'):
        file.write_text(file.read_text().replace('../../../../../target/tuple-names-libraries-v2/NeoCLR.ClassLibrary.props', str(props)))
    app = project / 'App/App.rvnproj'
    command = ['dotnet', str(args.compiler.resolve()), 'neoclr', '--project', str(app)]
    result = subprocess.run(command + ['--run', str(root / 'target/debug/neoclr')], capture_output=True, text=True)
    assert result.returncode == 42, result.stdout + result.stderr
    print('Combined union/intersection/named-tuple library consumer: 42')
    original = app.read_text()
    outputs = {p: p.read_bytes() for p in (project / 'App/bin').rglob('*.dll')}
    assert outputs, 'Expected native output'
    for setting, expected in [('false', 'tuple'), ('invalid', 'must be true or false')]:
        app.write_text(original.replace('<PropertyGroup>', '<PropertyGroup>\n<RavenNeoClrStructuralTuples>' + setting + '</RavenNeoClrStructuralTuples>', 1))
        result = subprocess.run(command, capture_output=True, text=True)
        assert result.returncode != 0, 'Reduced/invalid profile unexpectedly succeeded'
        assert expected in (result.stdout + result.stderr).lower(), result.stdout + result.stderr
        assert all(p.read_bytes() == content for p, content in outputs.items()), 'Failed build changed output'
        print(f'Tuple profile {setting}: rejected without changing app output')
