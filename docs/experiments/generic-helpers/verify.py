"""Verify bounded ordinary generic application import and retained constraint limits."""
import argparse
from pathlib import Path
import shutil
import subprocess
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
for name in ('runtime', 'bridge', 'system', 'reference'):
    parser.add_argument('--' + name, required=True, type=Path)
args = parser.parse_args()
artifacts = {name: getattr(args, name).resolve() for name in ('runtime', 'bridge', 'system', 'reference')}
here = Path(__file__).resolve().parent

def run(command):
    result = subprocess.run([str(part) for part in command], text=True, capture_output=True, timeout=180)
    assert result.returncode == 0, (command, result.stdout, result.stderr)
    return result.stdout

print(run(['dotnet', artifacts['bridge'], '--generic-helper-checks']), end='', flush=True)
with tempfile.TemporaryDirectory(prefix='neoclr-generic-helpers-') as directory:
    root = Path(directory)
    for name in ('Main.rvn', 'Contracts.rvnproj'):
        shutil.copyfile(here / name, root / name)
    shutil.copyfile(artifacts['reference'], root / 'NeoCLR.CoreProbe.dll')
    run(['dotnet', artifacts['bridge'], '--project', root / 'Contracts.rvnproj', root / 'out'])
    app = root / 'out/App.neoil'
    print(run([artifacts['runtime'], 'verify', app, '--system', artifacts['system']]), end='', flush=True)
    assert run([artifacts['runtime'], 'run', app, '--system', artifacts['system']]) == 'Generic helpers passed\n'
    print('Generic helpers: execution and reference/array identity passed', flush=True)
    for name, source, diagnostic in (
        ('number-extra-constraint', 'import System.*\nfunc Next<T>(x: T) -> T where T: Number, struct => x + T.One\nfunc Main() { Next<int>(1) }', 'Numeric specialization requires'),
        ('reference-constraint', 'import System.*\nfunc Read<T>(x: T) -> T where T: class => x\nfunc Main() { Read<string>("text") }', 'Numeric specialization requires'),
    ):
        (root / 'Main.rvn').write_text(source + '\n')
        rejected = subprocess.run(['dotnet', str(artifacts['bridge']), '--project', str(root / 'Contracts.rvnproj'), str(root / name)], text=True, capture_output=True, timeout=180)
        assert rejected.returncode != 0 and diagnostic in rejected.stderr, (name, rejected.stdout, rejected.stderr)
    print('Generic helpers: unsupported constraints remain rejected', flush=True)
