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
        ('Conventional operator names with inferred callbacks',
         (bridge / 'samples/library-query-names.rvn').read_text(),
         (bridge / 'samples/library-query-names.expected.txt').read_text()),
        ('String equality in interface query predicates', header + """
func Main() {
    let orders: List<string> = ArrayList<string>()
    orders.Add("test")
    orders.Add("2")
    let items = orders.Filter(x => x == "2")
    for item in items {
        WriteLine(item)
    }
    let expected = System.String.Concat("te", "st")
    for item in orders.Filter(x => x == expected) {
        WriteLine(item)
    }
    for item in orders.Filter(x => x != expected) {
        WriteLine(item)
    }
    if expected == "test" {
        WriteLine("equal")
    }
    if expected != "test" {
        WriteLine("wrong")
    }
    if "é" == System.String.Concat("", "é") {
        WriteLine("utf8")
    }
}
""", '2\ntest\n2\nequal\nutf8\n'),
        ('Option and Result terminals', (bridge / 'samples/library-query-terminals.rvn').read_text(),
         'Absent\nAbsent\n0\n42\nAbsent\nEmpty\n0\nMultiple\n42\nMultiple\n42\nAbsent\nSystem.String\nNo result\n42\nAbsent\nAbsent\n0\n0\nEmpty\nMultiple\n42\n'),
        ('Array queries and reflection materialization', (bridge / 'samples/library-array-queries.rvn').read_text(),
         'Parse\nDivide\nEquals\nToString\nCompareTo\n0\n17\n13\n3\n17\n52\n6\n0\n0\n43\n'),
        ('Array Iterable parameters, returns and independent iterators', header + '''
func Pass(values: int[]) -> Iterable<int> {
    return values
}
func First(values: Iterable<int>) -> int {
    let iterator = values.GetIterator()
    iterator.MoveNext()
    let result = iterator.Current
    iterator.Dispose()
    return result
}
func Main() {
    let values: int[] = [7, 42]
    let sequence: Iterable<int> = values
    WriteLine(First(values))
    WriteLine(First(Pass(values)))
    let first = sequence.GetIterator()
    let second = sequence.GetIterator()
    first.MoveNext()
    first.MoveNext()
    second.MoveNext()
    WriteLine(first.Current)
    WriteLine(second.Current)
    values[0] = 99
    WriteLine(second.Current)
    first.Dispose()
    second.Dispose()
    WriteLine(sequence.ToList()[0])
}
''', '7\n7\n42\n7\n99\n99\n'),
        ('Array query retained across GC', header + '''
func Make() -> Iterable<int> {
    let values: int[] = [7, 42]
    return values.Map((value: int) -> int => value + 1)
}
func Main() {
    let query = Make()
    var remaining = 1000
    while remaining != 0 {
        ArrayList<int>()
        remaining = remaining - 1
    }
    let result = query.ToList()
    WriteLine(result[0])
    WriteLine(result[1])
}
''', '8\n43\n'),
        ('Custom Raven Iterable and Iterator', (bridge / 'samples/application-iterable.rvn').read_text(), '0\n1\n1\n43\n2\n2\n'),
        ('Deferred callbacks and repeated enumeration', (bridge / 'samples/library-queries.rvn').read_text(),
         '0\n0\nForty two\nForty two\n2\n1\nDisposed\n2\nForty two\nNinety nine\n5\n3\n'),
        ('Empty and exhausted', header + '''
func Main() {
    let empty = ArrayList<int>()
    var calls = 0
    let query = empty.Filter((value: int) -> bool => {
        calls = calls + 1
        return true
    }).Map((value: int) -> int => {
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
    let query = values.Filter((value: int) -> bool => accept)
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
    let query = values.Map((value: int) -> int => value + offset)
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
    let copied = boxes.Filter((value: Box) -> bool => true).ToList()
    original.Value = 42
    WriteLine(copied[0].Value)
    copied.Add(Box(99))
    WriteLine(boxes.Count)
    let flags = boxes.Map((value: Box) -> bool => value.Value == 42).ToList()
    if flags[0] { WriteLine("Boolean") }
    let dates = ArrayList<Date>()
    dates.Add(Date.FromDayNumber(0).GetOkCase().Value)
    WriteLine(dates.Map((value: Date) -> int => value.DayNumber).ToList()[0])
    let visit: Func<Box, System.Void> = (value: Box) => { WriteLine(value.Value) }
    let units = boxes.Map(visit).ToList()
    WriteLine(units.Count)
}
''', '42\n1\nBoolean\n0\n42\n1\n'))
    # Exercise both expression results and conditional branches with the same
    # nonconstant operands. Python supplies the expected IEEE/integer ordering.
    operators = ('==', '!=', '<', '>', '<=', '>=')
    numeric_cases = (
        ('int', '-2147483648', -2147483648, '2147483647', 2147483647),
        ('long', '-9223372036854775808L', -9223372036854775808, '9223372036854775807L', 9223372036854775807),
        ('uint', '(uint)-1', 4294967295, '(uint)1', 1),
        ('ulong', '(ulong)-1L', 18446744073709551615, '(ulong)1L', 1),
        ('double', 'Math.Sqrt(-1.0)', float('nan'), '1.0', 1.0),
        ('double', '-0.0', -0.0, '0.0', 0.0),
        ('double', 'Math.Exp(1000.0)', float('inf'), '1.0', 1.0),
    )
    for index, (type_name, a_source, a, b_source, b) in enumerate(numeric_cases):
        checks = []
        expected = []
        for left_source, left, right_source, right in (
                ('a', a, 'b', b), ('b', b, 'a', a), ('a', a, 'a', a)):
            for op in operators:
                expression = f'{left_source} {op} {right_source}'
                checks.append(f'    Show({expression})')
                checks.append(f'    if {expression} {{\n        WriteLine(1)\n    }} else {{\n        WriteLine(0)\n    }}')
                answer = {'==': left == right, '!=': left != right,
                          '<': left < right, '>': left > right,
                          '<=': left <= right, '>=': left >= right}[op]
                expected.extend(['1' if answer else '0'] * 2)
        source = header + 'func Show(value: bool) {\n    if value {\n        WriteLine(1)\n    } else {\n        WriteLine(0)\n    }\n}\n'
        source += f'func Check(a: {type_name}, b: {type_name}) {{\n' + '\n'.join(checks) + '\n}\n'
        source += f'func Main() {{\n    Check({a_source}, {b_source})\n}}\n'
        cases.append((f'Numeric comparison boundary {index}: {type_name}', source, '\n'.join(expected) + '\n'))
    cases.append(('Mixed numeric comparison promotion', header + '''
func Greater(left: uint, right: int) -> bool {
    return left > right
}
func Main() {
    if Greater((uint)-1, 1) {
        WriteLine(1)
    }
    if Greater((uint)0, -1) {
        WriteLine(2)
    }
}
''', '1\n2\n'))
    cases.append(('Ordered query predicate', header + '''
func Main() {
    let values: int[] = [1, 2, 3]
    for value in values.Filter((value: int) -> bool => value > 1) {
        WriteLine(value)
    }
}
''', '2\n3\n'))
    from query_basic_cases import cases as basic_cases
    cases.extend(basic_cases())
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
    let iterator = values.Map((value: int) -> int => value).GetIterator()
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
    for operator in ('Take(1)', 'Skip(0)', 'Concat(values)',
                     'FlatMap((value: int) -> Iterable<int> => values)'):
        for state, advance in [('before advance', ''),
                               ('after exhaustion', 'while iterator.MoveNext() {}'),
                               ('after disposal', 'iterator.Dispose()')]:
            (root / 'Main.rvn').write_text(header + 'func Main() {\n'
                + '    let values: int[] = [42]\n'
                + '    let iterator = values.' + operator + '.GetIterator()\n'
                + '    ' + advance + '\n    WriteLine(iterator.Current)\n}\n')
            run = subprocess.run(command, capture_output=True, text=True, timeout=120)
            assert run.returncode != 0 and 'Query iterator has no current element' in run.stderr, run.stdout + run.stderr
            results[operator + ' Current ' + state] = 'faulted'
    (root / 'Main.rvn').write_text(header + '''
func Main() {
    let values = ArrayList<int>()
    values.Add(42)
    let query = values.Map((value: int) -> int => {
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
        ('Retired Where name', 'values.Where((value: int) -> bool => true)'),
        ('Retired Select name', 'values.Select((value: int) -> int => value)'),
        ('Indexed predicate overload', 'values.Filter((value: int, index: int) -> bool => true)'),
        ('Wrong predicate result', 'values.Filter((value: int) -> int => value)'),
        ('Unimplemented operator', 'values.OrderBy((value: int) -> int => value)')):
        (root / 'Main.rvn').write_text(header + 'func Main() { let values = ArrayList<int>()\n' + expression + '\n}')
        run = subprocess.run(command, capture_output=True, text=True, timeout=120)
        assert run.returncode != 0 and 'RAV' in run.stderr, run.stdout + run.stderr
        assert 'Verified saved project:' not in run.stdout
        results[label] = 'rejected before execution'
    for expression in ('left + right', 'left < right'):
        (root / 'Main.rvn').write_text(header + f'''
func Check(left: ulong, right: long) {{
    let invalid = {expression}
}}
func Main() {{
    Check((ulong)1L, 1L)
}}
''')
        run = subprocess.run(command, capture_output=True, text=True, timeout=120)
        assert run.returncode != 0 and 'RAV0024' in run.stderr, run.stdout + run.stderr
        assert 'Verified saved project:' not in run.stdout
        results[f'Mixed signed/unsigned rejected: {expression}'] = 'rejected before execution'
print(json.dumps(results, indent=2))
