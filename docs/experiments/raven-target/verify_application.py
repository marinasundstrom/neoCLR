"""Exercise application metadata through saved Raven source and the neoCLR verifier."""
import argparse
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
from runner_options import add_toolchain_arguments, runner_arguments

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('project', type=Path)
parser.add_argument('--runtime', type=Path, required=True)
add_toolchain_arguments(parser)
args = parser.parse_args()
bridge = Path(__file__).resolve().parent
results = {}
with tempfile.TemporaryDirectory(prefix='neoclr-application-check-') as temporary:
    root = Path(temporary)
    for name in ('Demo.rvnproj', 'NeoCLR.CoreProbe.dll'):
        shutil.copyfile(args.project.resolve().parent / name, root / name)
    command = [sys.executable, str(bridge / 'run_project.py'), str(root / 'Demo.rvnproj'),
               *runner_arguments(args), '--runtime', str(args.runtime.resolve())]
    source = (bridge / 'samples/application-types.rvn').read_text()
    for label, text, expected in [
        ('Application interfaces', (bridge / 'samples/application-interfaces.rvn').read_text(), '42\n99\n'),
        ('Abstract inheritance and overrides', (bridge / 'samples/application-inheritance.rvn').read_text(), '7\n42\n'),
        ('Class identity and value copies', source, '42\n99\n7\n42\n7\n'),
        ('Saved source rebuild', source.replace('counter.Set(42)', 'counter.Set(21)'), '21\n99\n7\n42\n7\n'),
    ]:
        (root / 'Main.rvn').write_text(text)
        run = subprocess.run(command, capture_output=True, text=True, timeout=120)
        if run.returncode or not run.stdout.endswith(expected):
            raise AssertionError(label + ': ' + run.stdout + run.stderr)
        results[label] = 'passed'
        artifact = max((root / '.neoclr-build').glob('*/output/App.neoil'), key=lambda p: p.stat().st_mtime_ns)
        lines = artifact.read_text().splitlines()
        mapping = json.loads(Path(str(artifact) + '.map.json').read_text())
        for entry in mapping['Mappings']:
            assert lines[entry['OutputLine'] - 1] == f"M{entry['MethodToken']:08x}_IL_{entry['Offset']:04x}:"
    for label, text in [
        ('Type initializer', source.replace('class Counter {', 'class Counter { static init { WriteLine("Unexpected") }')),
        ('Readonly field', source.replace('class Counter {', 'class Counter { readonly field Id: int = 1')),
    ]:
        (root / 'Main.rvn').write_text(text)
        run = subprocess.run(command, capture_output=True, text=True, timeout=120)
        if run.returncode == 0 or 'Verified saved project:' in run.stdout:
            raise AssertionError(label + ': invalid input ran or reused stale output')
        if 'Unsupported application' not in run.stderr:
            raise AssertionError(label + ': expected importer rejection: ' + run.stderr)
        results[label] = 'rejected before execution'
print(json.dumps(results, indent=2))
