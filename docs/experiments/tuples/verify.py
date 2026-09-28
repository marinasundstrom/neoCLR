"""Compile, admit and execute tuple consumers against matching development artifacts."""
import argparse
import json
from pathlib import Path
import shutil
import subprocess
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
for name in ('runtime', 'bridge', 'system', 'reference'):
    parser.add_argument('--' + name, type=Path, required=True)
parser.add_argument('--evidence', type=Path)
args = parser.parse_args()
tools = {name: getattr(args, name).resolve() for name in ('runtime', 'bridge', 'system', 'reference')}
here = Path(__file__).resolve().parent
cases = [('syntax-copy-nesting', (here / 'Main.rvn').read_text(), 'Tuple contracts passed\n')]
for n in range(1, 8):
    arguments = ', '.join(str(i) for i in range(1, n + 1))
    types = ', '.join('int' for _ in range(n))
    source = f'''import System.*
func Main() -> int {{
    let value = Tuple<{types}>({arguments})
    if value.Item{n} != {n} {{ return 1 }}
    let zero: Tuple<{types}> = default
    if zero.Item{n} != 0 {{ return 2 }}
    return 0
}}
'''
    cases.append((f'arity-{n}-construction-default', source, ''))
cases.append(('boxing-array', '''import System.*
func Main() -> int {
    let original = (42, "kept")
    let boxed: Object = original
    System.Runtime.GC.Collect()
    let restored = (Tuple<int, string>)boxed
    if restored.Item1 != 42 || restored.Item2 != "kept" { return 1 }
    let values = [original, (7, "second")]
    if values[1].Item1 != 7 { return 2 }
    return 0
}
''', ''))
results = []
def run(command):
    result = subprocess.run([str(x) for x in command], capture_output=True, text=True, timeout=180)
    assert result.returncode == 0, (command, result.returncode, result.stdout, result.stderr)
    return result
with tempfile.TemporaryDirectory(prefix='neoclr-tuples-') as directory:
    root = Path(directory)
    shutil.copyfile(here.parent / 'entry-results/Contracts.rvnproj', root / 'Contracts.rvnproj')
    shutil.copyfile(tools['reference'], root / 'NeoCLR.CoreProbe.dll')
    for name, source, expected in cases:
        (root / 'Main.rvn').write_text(source)
        build = root / name
        run(['dotnet', tools['bridge'], '--project', root / 'Contracts.rvnproj', build])
        run(['dotnet', tools['bridge'], '--tuple-image-checks', build / 'App.dll'])
        app = build / 'App.neoil'
        run([tools['runtime'], 'verify', app, '--system', tools['system']])
        executed = run([tools['runtime'], 'run', app, '--system', tools['system']])
        assert executed.stdout == expected and not executed.stderr, (name, executed.stdout, executed.stderr)
        results.append({'case': name, 'status': 'passed'})
        print(name + ': passed', flush=True)
if args.evidence:
    args.evidence.write_text(json.dumps({'cases': results}, indent=2) + '\n')
