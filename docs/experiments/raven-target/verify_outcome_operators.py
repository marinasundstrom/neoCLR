"""Verify the bounded Option/Result operator port with the saved-project compiler."""
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
add_toolchain_arguments(parser)
parser.add_argument('--runtime', type=Path, required=True)
args = parser.parse_args()
bridge = Path(__file__).resolve().parent
header = '''import System.*
import System.Option.*
import System.Result.*
import System.Linq.*
import System.Console.*
'''
results = {}
with tempfile.TemporaryDirectory(prefix='neoclr-outcomes-') as temporary:
    root = Path(temporary)
    for name in ('Demo.rvnproj', 'NeoCLR.CoreProbe.dll'):
        shutil.copyfile(args.project.resolve().parent / name, root / name)
    command = [sys.executable, str(bridge / 'run_project.py'), str(root / 'Demo.rvnproj'),
               *runner_arguments(args), '--runtime', str(args.runtime.resolve())]

    def run(source):
        (root / 'Main.rvn').write_text(source)
        return subprocess.run(command, capture_output=True, text=True, timeout=120)

    cases = [
        ('Website example', (bridge / 'samples/library-outcome-operators.rvn').read_text(),
         (bridge / 'samples/library-outcome-operators.expected.txt').read_text()),
        ('All overloads and branch selection',
         (bridge / 'samples/library-outcome-operator-checks.rvn').read_text(),
         'Option operators passed\nResult operators passed\n'),
        ('Reference and completion payloads', header + '''
func Main() {
    let word: Option<string> = Some("text")
    let mapped = word.Map(value => String.Concat(value, "!"))
    match mapped {
        Some(let value) => WriteLine(value)
        None => System.Fault("Lost reference payload")
    }
    let completed: Result<unit, string> = Ok(())
    let answer = completed.Map((value: unit) -> int => 42)
    WriteLine(answer.UnwrapOr(0))
    let absent: Option<unit> = None
    WriteLine(absent.ToIterable().Count())
    let nested: Option<Option<string>> = Some(word)
    WriteLine(nested.Flatten().UnwrapOr("missing"))
}
''', 'text!\n42\n0\ntext\n')]
    for label, source, expected in cases:
        result = run(source)
        assert result.returncode == 0, result.stdout + result.stderr
        lines = result.stdout.splitlines()
        marker = next(i for i, line in enumerate(lines) if line.startswith('Verified saved project:'))
        actual = '\n'.join(lines[marker + 1:]) + '\n'
        assert actual == expected, (label, actual, expected)
        results[label] = 'passed'

    result = run(header + '''
func Main() {
    let value: Option<int> = Some(1)
    value.Map((item: int) -> int => {
        System.Fault("Outcome callback failed")
        return item
    })
    WriteLine("Must not continue")
}
''')
    assert result.returncode != 0 and 'Outcome callback failed' in result.stderr, result.stdout + result.stderr
    assert 'Must not continue' not in result.stdout
    results['Callback faults remain terminal'] = 'passed'
    for label, expression in (
        ('Wrong predicate result', 'value.Filter((item: int) -> int => item)'),
        ('Wrong receiver', '42.TapNone(() => {})'),
        ('Then requires Option result', 'value.Then((item: int) -> int => item)'),
        ('Flatten requires nested Option', 'value.Flatten()'),
        ('No Where alias', 'value.Where((item: int) -> bool => true)')):
        result = run(header + 'func Main() {\n    let value: Option<int> = Some(1)\n    ' + expression + '\n}\n')
        assert result.returncode != 0 and 'RAV' in result.stderr, result.stdout + result.stderr
        assert 'Verified saved project:' not in result.stdout
        results[label] = 'rejected before execution'
print(json.dumps(results, indent=2))
