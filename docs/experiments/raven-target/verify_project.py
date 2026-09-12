"""Check saved-source execution and rejection without stale-artifact fallback."""
import argparse
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('project', type=Path)
parser.add_argument('--raven', required=True, type=Path)
parser.add_argument('--runtime', required=True, type=Path)
args = parser.parse_args()
bridge = Path(__file__).resolve().parent
results = {}
with tempfile.TemporaryDirectory(prefix='neoclr-project-check-') as temporary:
    root = Path(temporary)
    for name in ('Demo.rvnproj', 'NeoCLR.CoreProbe.dll'):
        shutil.copyfile(args.project.resolve().parent / name, root / name)
    command = [sys.executable, str(bridge / 'run_project.py'), str(root / 'Demo.rvnproj'),
               '--raven', str(args.raven.resolve()), '--runtime', str(args.runtime.resolve())]
    cases = [('Result', 'library-result.rvn', '42\nOverflow\n=> Void\n'),
             ('Option', 'library-option.rvn', '42\nProduct not found\n=> Void\n'),
             ('Void', 'library-void.rvn', 'Completed without a payload\nNot completed\n=> Void\n')]
    for label, sample, expected in cases:
        (root / 'Main.rvn').write_text((bridge / 'samples' / sample).read_text())
        run = subprocess.run(command, capture_output=True, text=True, timeout=90)
        if run.returncode or not run.stdout.endswith(expected):
            raise AssertionError(run.stdout + run.stderr)
        results[label] = expected
    source = (bridge / 'samples/library-result.rvn').read_text().replace('Show(-42)', 'Show(-7)')
    (root / 'Main.rvn').write_text(source)
    run = subprocess.run(command, capture_output=True, text=True, timeout=90)
    if run.returncode or not run.stdout.endswith('7\nOverflow\n=> Void\n'):
        raise AssertionError(run.stdout + run.stderr)
    results['SavedEdit'] = '7\nOverflow\n=> Void\n'
    for label, source, diagnostic in [
        ('CompileFailure', 'func Main() { MissingCall() }', 'RAV'),
        ('ImportFailure', 'func Add(value: int) -> int { return value + 1 }\nfunc Main() { Add(2) }', 'Unsupported')]:
        before = set(root.rglob('App.neoil'))
        (root / 'Main.rvn').write_text(source)
        run = subprocess.run(command, capture_output=True, text=True, timeout=90)
        if run.returncode == 0 or diagnostic not in run.stderr or set(root.rglob('App.neoil')) != before:
            raise AssertionError(run.stdout + run.stderr)
        results[label] = 'Rejected; no executable produced or stale output run'
print(json.dumps(results, indent=2))
