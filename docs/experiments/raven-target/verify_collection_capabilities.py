"""Verify unavailable collection capabilities fail before execution."""
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
results = {}
with tempfile.TemporaryDirectory(prefix='neoclr-capabilities-') as temporary:
    root = Path(temporary)
    for name in ('Demo.rvnproj', 'NeoCLR.CoreProbe.dll'):
        shutil.copyfile(args.project.resolve().parent / name, root / name)
    command = [sys.executable, str(Path(__file__).with_name('run_project.py')), str(root / 'Demo.rvnproj'),
               *runner_arguments(args), '--runtime', str(args.runtime.resolve())]
    for name, source in {
        'read indexer cannot replace': 'func Change(values: Sequence<int>) { values[0] = 42 }',
        'read sequence cannot grow': 'func Change(values: Sequence<int>) { values.Add(42) }',
        'replacement does not imply growth': 'func Change(values: MutableSequence<int>) { values.Add(42) }',
        'array cannot satisfy growth parameter': 'func Grow(values: List<int>) { values.Add(42) }\nfunc Main() { let values: int[] = [1]\nGrow(values) }',
        'collection has no indexer': 'func Read(values: Collection<int>) -> int { return values[0] }',
        'read sequence remains invariant': 'func Widen(values: Sequence<string>) -> Sequence<Object> { return values }',
    }.items():
        if 'func Main' not in source:
            source += '\nfunc Main() {}'
        (root / 'Main.rvn').write_text('import System.*\nimport System.Collections.*\n' + source)
        result = subprocess.run(command, capture_output=True, text=True, timeout=90)
        assert result.returncode != 0 and 'error RAV' in result.stderr + result.stdout, result.stdout + result.stderr
        assert not list(root.rglob('App.neoil')), name
        results[name] = 'compiler rejected; no executable produced'
print(json.dumps(results, indent=2))
