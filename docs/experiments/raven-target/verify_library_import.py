"""Compile independent Raven libraries and a consumer, then audit/import/execute."""
import argparse
import json
import re
from pathlib import Path
import shutil
import subprocess
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('project', type=Path)
for name in ('compiler', 'bridge', 'runtime', 'system'):
    parser.add_argument('--' + name, type=Path, required=True)
args = parser.parse_args()
project_text = args.project.resolve().read_text()
results = {}

def run(command):
    return subprocess.run([str(x) for x in command], text=True, capture_output=True, timeout=120)

def require(result, label):
    assert result.returncode == 0, label + ': ' + result.stdout + result.stderr

with tempfile.TemporaryDirectory(prefix='neoclr-library-import-') as temporary:
    root = Path(temporary)
    core = root / 'NeoCLR.CoreProbe.dll'
    shutil.copyfile(args.project.resolve().parent / core.name, core)

    def compile(name, source, references=(), library=False, identity=None):
        identity = identity or name
        folder = root / name
        folder.mkdir()
        shutil.copyfile(core, folder / core.name)
        text = project_text.replace('<OutputType>Exe</OutputType>', '<OutputType>Library</OutputType>' if library else '<OutputType>Exe</OutputType>')
        text = text.replace('<PropertyGroup>', '<PropertyGroup>\n<AssemblyName>' + identity + '</AssemblyName>', 1)
        for path in references:
            shutil.copyfile(path, folder / path.name)
            text = text.replace('</ItemGroup>', f'<Reference Include="{path.stem}"><HintPath>{path.name}</HintPath></Reference></ItemGroup>', 1)
        (folder / 'Demo.rvnproj').write_text(text)
        (folder / 'Main.rvn').write_text(source)
        result = run(['dotnet', args.compiler.resolve(), folder / 'Demo.rvnproj', '--framework', 'net11.0', '--no-project-restore', '-o', folder / 'compiled'])
        require(result, name)
        return folder / 'compiled' / (identity + '.dll')

    arithmetic_source = '''public class Arithmetic {
    public static func Sum(value: int) -> int {
        if value == 0 {
            return 0
        }
        return value + Sum(value - 1)
    }
}
'''
    arithmetic = compile('ArithmeticLibrary', arithmetic_source, library=True)
    facade = compile('FacadeLibrary', '''namespace Workflows

public func Answer() -> int {
    return Arithmetic.Sum(8) + 6
}

public class Facade {
    public static func Forward() -> int {
        return Answer()
    }
}
''', [arithmetic], library=True)
    consumer = compile('Consumer', '''import System.Console.*
import Workflows.*
func Main() {
    WriteLine(Answer())
    WriteLine(Arithmetic.Sum(3))
}
''', [arithmetic, facade])

    def import_case(name, libraries, success=True, diagnostic=None, expected="42\n6\n", gc=False):
        output = root / name
        result = run(['dotnet', args.bridge.resolve(), '--import', consumer, core, output, *libraries])
        if not success:
            assert result.returncode != 0 and not (output / 'App.neoil').exists(), result.stdout + result.stderr
            if diagnostic:
                assert diagnostic in result.stdout + result.stderr, result.stdout + result.stderr
            results[name] = 'rejected without executable output'
            return
        require(result, name)
        require(run([args.runtime.resolve(), 'verify', output / 'App.neoil', '--system', args.system.resolve()]), name)
        executed = run([args.runtime.resolve(), 'run', output / 'App.neoil', '--system', args.system.resolve(), *(['--gc-stats'] if gc else [])])
        require(executed, name)
        assert executed.stdout == expected, executed.stdout
        if gc:
            assert int(re.search(r'collections=(\d+)', executed.stderr).group(1)) > 0, executed.stderr
        mapping = json.loads((output / 'App.neoil.map.json').read_text())
        identities = mapping['MethodIdentities']
        assert len({m['AssemblyIdentity'] for m in identities}) == 3
        # Distinct assemblies normally reuse the same MethodDef rows. Both must run.
        assert len({m['MethodToken'] for m in identities}) < len(identities)
        lines = (output / 'App.neoil').read_text().splitlines()
        assert all(lines[m['OutputLine'] - 1] == m['OutputLabel'] for m in mapping['Mappings'])
        results[name] = 'separately compiled, imported, verified and executed; qualified source maps checked'

    import_case('transitive-libraries', [arithmetic, facade])
    import_case('reverse-input-order', [facade, arithmetic])
    import_case('missing-library', [facade], False, 'unresolved')
    import_case('duplicate-library', [arithmetic, facade, arithmetic], False, 'Duplicate assembly identity')
    private = compile('PrivateImplementation', arithmetic_source.replace('public static func Sum', 'private static func Sum'), library=True, identity='ArithmeticLibrary')
    import_case('private-implementation', [private, facade], False, 'Nonpublic cross-type call')
    changed = compile('ChangedSignature', arithmetic_source.replace('Sum(value: int)', 'Sum(value: int, unused: int)').replace('Sum(value - 1)', 'Sum(value - 1, 0)'), library=True, identity='ArithmeticLibrary')
    import_case('changed-signature', [changed, facade], False, 'unresolved')
    unsupported = compile('UnsupportedBody', arithmetic_source.replace('return value + Sum(value - 1)', 'return Counter().Read()') + '\npublic class Counter { public func Read() -> int { return 42 } }\n', library=True, identity='ArithmeticLibrary')
    import_case('library-instance-body', [unsupported, facade], expected='48\n42\n')

    base_source = '''public interface Reader {
    func Read() -> int
}
public abstract class CounterBase : Reader {
    public var Age: int
    public init(age: int) { Age = age }
    public abstract func Read() -> int
    public virtual func Set(age: int) { Age = age }
}
public struct Point {
    public var X: int
    public init(x: int) { X = x }
    public func Set(x: int) { X = x }
}
'''
    base = compile('BaseLibrary', base_source, library=True)
    derived = compile('DerivedLibrary', '''public class Counter : CounterBase {
    public init(age: int) : base(age) { }
    public override func Read() -> int { return Age }
    public override func Set(age: int) { base.Set(age) }
}
''', [base], library=True)
    consumer = compile('InstanceConsumer', '''import System.*
import System.Collections.*
import System.Console.*
func Main() {
    let counter = Counter(7)
    let parent: CounterBase = counter
    let reader: Reader = parent
    WriteLine(reader.Read())
    parent.Set(42)
    WriteLine(counter.Age)
    let read: Func<int> = reader.Read
    WriteLine(read())
    var point = Point(7)
    var copy = point
    copy.Set(99)
    WriteLine(point.X)
    WriteLine(copy.X)
    let values = ArrayList<CounterBase>()
    values.Add(counter)
    values[0].Set(21)
    var remaining = 100
    while remaining != 0 {
        Counter(remaining)
        remaining = remaining - 1
    }
    WriteLine(read())
}
''', [base, derived])
    instance_expected = '7\n42\n42\n7\n99\n21\n'
    import_case('library-inheritance-interface-values-delegate', [base, derived], expected=instance_expected, gc=True)
    import_case('library-types-reversed-inputs', [derived, base], expected=instance_expected)
    hidden = compile('HiddenBase', base_source.replace('public abstract class CounterBase', 'internal abstract class CounterBase'), library=True, identity='BaseLibrary')
    import_case('hidden-library-type', [hidden, derived], False, 'Nonpublic imported type')
    private_ctor = compile('PrivateConstructor', base_source.replace('public init(age:', 'private init(age:'), library=True, identity='BaseLibrary')
    import_case('private-library-base-constructor', [private_ctor, derived], False, 'Nonpublic cross-type call')

    generic = compile('GenericBody', base_source.replace('public func Set(x: int)', 'public func Identity<T>(value: T) -> T { return value }\n    public func Set(x: int)'), library=True, identity='BaseLibrary')
    import_case('generic-library-body-boundary', [generic, derived], False, 'Unsupported application signature')
    collision_source = '''namespace Shared {
    public class Item { public func Read() -> int { return NUMBER } }
}
namespace API {
    public func Read() -> int { return Shared.Item().Read() }
}
'''
    left = compile('LeftLibrary', collision_source.replace('NUMBER', '42').replace('API', 'Left'), library=True)
    right = compile('RightLibrary', collision_source.replace('NUMBER', '99').replace('API', 'Right'), library=True)
    consumer = compile('CollisionConsumer', '''import System.Console.*
func Main() {
    WriteLine(Left.Read())
    WriteLine(Right.Read())
}
''', [left, right])
    import_case('same-type-name-different-assemblies', [left, right], expected='42\n99\n')

print(json.dumps(results, indent=2))
