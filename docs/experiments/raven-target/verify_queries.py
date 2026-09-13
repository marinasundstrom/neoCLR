"""Verify deferred library queries, ownership, cached values and fault boundaries."""
import argparse
import json
from pathlib import Path
import re
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
header = 'import System.*\nimport System.Collections.*\nimport System.Linq.*\nimport System.Console.*\n'
results = {}
with tempfile.TemporaryDirectory(prefix='neoclr-queries-') as temporary:
    root = Path(temporary)
    for name in ('Demo.rvnproj', 'NeoCLR.CoreProbe.dll'):
        shutil.copyfile(args.project.resolve().parent / name, root / name)
    command = [sys.executable, str(bridge / 'run_project.py'), str(root / 'Demo.rvnproj'),
               *runner_arguments(args), '--runtime', str(args.runtime.resolve())]
    cases = [
        ('Custom Raven Iterable and Iterator', (bridge / 'samples/application-iterable.rvn').read_text(), '0\n1\n1\n43\n2\n2\n'),
        ('Deferred callbacks and repeated enumeration', (bridge / 'samples/library-queries.rvn').read_text(),
         '0\n0\nForty two\nForty two\n2\n1\nDisposed\n2\nForty two\nNinety nine\n5\n3\n'),
        ('Empty and exhausted', header + '''
func Main() {
    let empty = ArrayList<int>()
    var calls = 0
    let query = empty.Where((value: int) -> bool => {
        calls = calls + 1
        return true
    }).Select((value: int) -> int => {
        calls = calls + 1
        return value
    })
    WriteLine(query.ToList().Count)
    let iterator = query.GetIterator()
    if !iterator.MoveNext() { WriteLine("Empty") }
    if !iterator.MoveNext() { WriteLine("Still empty") }
    iterator.Dispose()
    WriteLine(calls)
}
''', '0\nEmpty\nStill empty\n0\n'),
        ('All filtered and mutable captures', header + '''
func Main() {
    let values = ArrayList<int>()
    values.Add(7)
    var accept = false
    let query = values.Where((value: int) -> bool => accept)
    WriteLine(query.ToList().Count)
    accept = true
    values.Add(42)
    let materialized = query.ToList()
    WriteLine(materialized.Count)
    values[0] = 99
    WriteLine(materialized[0])
    WriteLine(query.ToList()[0])
}
''', '0\n2\n7\n99\n'),
        ('Escaping query across GC', header + '''
func Make() -> Iterable<int> {
    let values = ArrayList<int>()
    values.Add(7)
    var offset = 1
    let query = values.Select((value: int) -> int => value + offset)
    offset = 2
    return query
}
func Main() {
    let query = Make()
    var remaining = 1000
    while remaining != 0 {
        ArrayList<int>()
        remaining = remaining - 1
    }
    WriteLine(query.ToList()[0])
}
''', '9\n'),
    ]
    cases.append(('Reference, value and Void payloads', '''
import System.*
import System.Collections.*
import System.Linq.*
import System.Console.*
class Box {
    var Value: int
    init(value: int) { Value = value }
}
func Main() {
    let boxes = ArrayList<Box>()
    let original = Box(7)
    boxes.Add(original)
    let copied = boxes.Where((value: Box) -> bool => true).ToList()
    original.Value = 42
    WriteLine(copied[0].Value)
    copied.Add(Box(99))
    WriteLine(boxes.Count)
    let flags = boxes.Select((value: Box) -> bool => value.Value == 42).ToList()
    if flags[0] { WriteLine("Boolean") }
    let dates = ArrayList<Date>()
    dates.Add(Date.FromDayNumber(0).GetOkCase().Value)
    WriteLine(dates.Select((value: Date) -> int => value.DayNumber).ToList()[0])
    let visit: Func<Box, System.Void> = (value: Box) => { WriteLine(value.Value) }
    let units = boxes.Select(visit).ToList()
    WriteLine(units.Count)
}
''', '42\n1\nBoolean\n0\n42\n1\n'))
    for label, source, expected in cases:
        (root / 'Main.rvn').write_text(source)
        run = subprocess.run(command, capture_output=True, text=True, timeout=120)
        assert run.returncode == 0 and run.stdout.endswith(expected), label + ': ' + run.stdout + run.stderr
        results[label] = 'passed'
        artifact = max((root / '.neoclr-build').glob('*/output/App.neoil'), key=lambda p: p.stat().st_mtime_ns)
        system = artifact.parent / 'System.Collections.neoil'
        if label == 'Escaping query across GC':
            collected = subprocess.run([str(args.runtime.resolve()), 'run', str(artifact), '--system', str(system), '--gc-stats'],
                                       capture_output=True, text=True, timeout=120)
            assert collected.returncode == 0 and collected.stdout == expected, collected.stdout + collected.stderr
            assert int(re.search(r'collections=(\d+)', collected.stderr).group(1)) > 0, collected.stderr
    for command_name in ('verify', 'run'):
        run = subprocess.run([str(args.runtime.resolve()), command_name, str(bridge / 'samples/query-lifetime.neoil'), '--system', str(system)],
                             capture_output=True, text=True, timeout=120)
        assert run.returncode == 0, run.stdout + run.stderr
    results['Underlying iterator acquired and disposed once'] = 'passed'
    for label, advance in [('Current before MoveNext', ''),
                           ('Current after exhaustion', 'while iterator.MoveNext() {}'),
                           ('Current after Dispose', 'iterator.Dispose()')]:
        source = header + '''
func Main() {
    let values = ArrayList<int>()
    values.Add(42)
    let iterator = values.Select((value: int) -> int => value).GetIterator()
''' + advance + '''
    WriteLine(iterator.Current)
    WriteLine("Must not continue")
}
'''
        (root / 'Main.rvn').write_text(source)
        run = subprocess.run(command, capture_output=True, text=True, timeout=120)
        assert run.returncode != 0 and 'Query iterator has no current element' in run.stderr, run.stdout + run.stderr
        assert 'Must not continue' not in run.stdout
        results[label] = 'faulted'
    (root / 'Main.rvn').write_text(header + '''
func Main() {
    let values = ArrayList<int>()
    values.Add(42)
    let query = values.Select((value: int) -> int => {
        Result<int, Error>.Ok(value).GetErrorCase()
        return value
    })
    WriteLine("Deferred")
    query.ToList()
    WriteLine("Must not continue")
}
''')
    run = subprocess.run(command, capture_output=True, text=True, timeout=120)
    assert run.returncode != 0 and 'Fault:' in run.stderr, run.stdout + run.stderr
    assert 'Deferred\n' in run.stdout and 'Must not continue' not in run.stdout
    results['Callback fault is terminal and deferred'] = 'passed'
    for label, expression in (
        ('Indexed predicate overload', 'values.Where((value: int, index: int) -> bool => true)'),
        ('Wrong predicate result', 'values.Where((value: int) -> int => value)'),
        ('Unimplemented operator', 'values.OrderBy((value: int) -> int => value)')):
        (root / 'Main.rvn').write_text(header + 'func Main() { let values = ArrayList<int>()\n' + expression + '\n}')
        run = subprocess.run(command, capture_output=True, text=True, timeout=120)
        assert run.returncode != 0 and 'RAV' in run.stderr, run.stdout + run.stderr
        assert 'Verified saved project:' not in run.stdout
        results[label] = 'rejected before execution'
print(json.dumps(results, indent=2))
