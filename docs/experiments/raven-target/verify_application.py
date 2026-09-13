"""Exercise application metadata through saved Raven source and the neoCLR verifier."""
import argparse
import json
import re
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
    orders = (bridge / 'samples/application-order-collections.rvn').read_text()
    orders_expected = (bridge / 'samples/application-order-collections.expected.txt').read_text()
    for label, text, expected in [
        ('Order collections workflow', orders, orders_expected),
        ('Order collections with hash collisions', orders.replace('return number\n', 'return 0\n'), orders_expected),
        ('Extension receivers and callbacks', (bridge / 'samples/application-extensions.rvn').read_text(), '43\n41\n2\n7\n42\n2\n2\n'),
        ('Delegates and shared captures', (bridge / 'samples/application-delegates.rvn').read_text(), '8\n42\n99\n12\n15\n42\n42\n123\n123\n1\n-2147483648\n'),
        ('Application interfaces', (bridge / 'samples/application-interfaces.rvn').read_text(), '42\n99\n'),
        ('Abstract inheritance and overrides', (bridge / 'samples/application-inheritance.rvn').read_text(), '7\n42\n'),
        ('Class identity and value copies', source, '42\n99\n7\n42\n7\n'),
        ('Saved source rebuild', source.replace('counter.Set(42)', 'counter.Set(21)'), '21\n99\n7\n42\n7\n'),
    ]:
        if label == 'Delegates and shared captures':
            text = text.replace('func Main() {', 'func AllocateNoise() { var remaining = 100; while remaining != 0 { Counter(remaining); remaining = remaining - 1 } }\nfunc Main() {')
            text = text.replace('let next = MakeCounter(10)', 'let next = MakeCounter(10)\n    AllocateNoise()')
        if label == 'Order collections workflow':
            text = text.replace('func Main() {', 'func AllocateOrderNoise() { var remaining = 100; while remaining != 0 { Order(remaining, false); remaining = remaining - 1 } }\nfunc Main() {')
            text = text.replace('    PrintPending(orders)', '    AllocateOrderNoise()\n    PrintPending(orders)')
        (root / 'Main.rvn').write_text(text)
        run = subprocess.run(command, capture_output=True, text=True, timeout=120)
        if run.returncode or not run.stdout.endswith(expected):
            raise AssertionError(label + ': ' + run.stdout + run.stderr)
        results[label] = 'passed'
        artifact = max((root / '.neoclr-build').glob('*/output/App.neoil'), key=lambda p: p.stat().st_mtime_ns)
        if label in ('Delegates and shared captures', 'Order collections workflow'):
            collected = subprocess.run([str(args.runtime.resolve()), 'run', str(artifact), '--system',
                str(artifact.parent / 'System.Collections.neoil'), '--gc-stats'], capture_output=True, text=True, timeout=120)
            assert collected.returncode == 0 and collected.stdout == expected, collected.stdout + collected.stderr
            assert int(re.search(r'collections=(\d+)', collected.stderr).group(1)) > 0, collected.stderr
            results['Escaping closure across GC' if label == 'Delegates and shared captures' else 'Order collections across GC'] = 'passed'
        lines = artifact.read_text().splitlines()
        mapping = json.loads(Path(str(artifact) + '.map.json').read_text())
        for entry in mapping['Mappings']:
            assert lines[entry['OutputLine'] - 1] == f"M{entry['MethodToken']:08x}_IL_{entry['Offset']:04x}:"
    for label, text in [
        ('Type initializer', source.replace('class Counter {', 'class Counter { static init { WriteLine("Unexpected") }')),
        ('Readonly field', source.replace('class Counter {', 'class Counter { readonly field Id: int = 1')),
        ('Generic extension boundary', 'extension IdentityOperations<T> for T { func Identity() -> T { return self } }\nfunc Main() { System.Console.WriteLine(42.Identity()) }'),
    ]:
        (root / 'Main.rvn').write_text(text)
        run = subprocess.run(command, capture_output=True, text=True, timeout=120)
        if run.returncode == 0 or 'Verified saved project:' in run.stdout:
            raise AssertionError(label + ': invalid input ran or reused stale output')
        if 'Unsupported application' not in run.stderr:
            raise AssertionError(label + ': expected importer rejection: ' + run.stderr)
        results[label] = 'rejected before execution'
    (root / 'Main.rvn').write_text("""
import System.*
import System.Console.*
interface Reader { func Read() -> int }
class Holder { var Reader: Reader }
func Main() {
    let holder = Holder()
    let callback: Func<int> = holder.Reader.Read
    WriteLine("Must not reach invocation")
}
""")
    run = subprocess.run(command, capture_output=True, text=True, timeout=120)
    assert run.returncode != 0 and 'null delegate receiver' in run.stderr, run.stdout + run.stderr
    assert 'Must not reach invocation' not in run.stdout
    results['Null interface method group'] = 'faulted at binding'
    # Extension marker metadata is present for closure auditing, not an executable
    # exception API. A reachable constructor must still fail before execution.
    (root / 'Main.rvn').write_text('func Main() { System.NotImplementedException() }')
    run = subprocess.run(command, capture_output=True, text=True, timeout=120)
    assert run.returncode != 0 and 'Unsupported Result profile type: System.NotImplementedException' in run.stderr, run.stdout + run.stderr
    assert 'Verified saved project:' not in run.stdout
    results['Metadata-only marker dependency'] = 'rejected before execution'
print(json.dumps(results, indent=2))
