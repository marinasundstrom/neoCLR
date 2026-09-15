"""Execute a generic Raven class against an independent reference declaration."""
import argparse
from pathlib import Path
import subprocess
import tempfile
from xml.sax.saxutils import escape
from build_runtime_library import ROOT
from collection_library import build

parser = argparse.ArgumentParser(description=__doc__)
for name in ('compiler', 'bridge', 'runtime'):
    parser.add_argument('--' + name, required=True, type=Path)
parser.add_argument("--checked-storage", action="store_true")
parser.add_argument("--private-storage", action="store_true")
parser.add_argument("--private-methods", action="store_true")
args = parser.parse_args()
compiler, bridge, runtime = (getattr(args, n).resolve() for n in ('compiler', 'bridge', 'runtime'))


def run(command, success=True):
    result = subprocess.run([str(x) for x in command], capture_output=True, text=True, timeout=120)
    assert (result.returncode == 0) == success, result.stdout + result.stderr
    return result.stdout + result.stderr


with tempfile.TemporaryDirectory(prefix='neoclr-generic-instance-') as temporary:
    root = Path(temporary)
    (root / 'demo').mkdir()
    core = root / 'demo/NeoCLR.CoreProbe.dll'
    run(['dotnet', bridge, '--generic-instance-library-core', core])
    source = '''namespace Probe
import System.Collections.*
public class Cell<T> : Iterable<T> {
    private field stored: T
    public init(value: T) {
        stored = value
    }
    public func Set(value: T) {
        stored = value
    }
    public func CopyFrom(other: Cell<T>) {
        Set(other.Value)
    }
    public func Self() -> Cell<T> {
        return self
    }
    public val Value: T {
        get {
            let value: T = stored
            return value
        }
    }
    public func AsIterable() -> Iterable<T> {
        return self
    }
    public func Copy() -> Cell<T> {
        return Cell<T>(stored)
    }
    public func GetIterator() -> Iterator<T> {
        let values = ArrayList<T>()
        values.Add(stored)
        return values.GetIterator()
    }
}
'''

    if args.private_methods:
        source = source.replace('public func Set(value: T) {\n        stored = value',
                                'public func Set(value: T) {\n        Store(value)')
        source = source.replace('let value: T = stored', 'let value: T = Read()')
        source = source.rstrip()[:-1] + '''
    private func Store(value: T) { stored = value }
    private func Read() -> T { return stored }
}
'''

    if args.private_storage:
        source = source.replace('private field stored: T', 'private field stored: Storage<T>')
        source = source.replace('stored = value', 'stored.Set(value)')
        source = source.replace('public init(value: T) {\n        stored.Set(value)', 'public init(value: T) {\n        stored = Storage<T>(value)')
        source = source.replace('let value: T = stored', 'let value: T = stored.Read()')
        source = source.replace('Cell<T>(stored)', 'Cell<T>(stored.Read())').replace('values.Add(stored)', 'values.Add(stored.Read())')
        source += '\n' + '\n'.join([
            'internal class Storage<T> {',
            '    private field items: T[]',
            '    public init(value: T) {',
            '        items = System.Runtime.CompilerServices.CheckedStorage.Reserve<T>(1)',
            '        items[0] = value',
            '    }',
            '    public func Set(value: T) { items[0] = value }',
            '    public func Read() -> T { return items[0] }',
            '}',
        ])
    elif args.checked_storage:
        source = source.replace('private field stored: T', 'private field stored: T[]')
        source = source.replace('stored = value', 'stored[0] = value')
        source = source.replace('public init(value: T) {', 'public init(value: T) {\n        stored = System.Runtime.CompilerServices.CheckedStorage.Reserve<T>(1)')
        source = source.replace('let value: T = stored', 'let value: T = stored[0]')
        source = source.replace('Cell<T>(stored)', 'Cell<T>(stored[0])').replace('values.Add(stored)', 'values.Add(stored[0])')

    def compile(name, text, success=True):
        folder = root / name
        folder.mkdir()
        (folder / 'Main.rvn').write_text(text)
        project = folder / 'Library.rvnproj'
        project.write_text(f'''<Project><PropertyGroup><OutputType>Library</OutputType>
<AssemblyName>{name}</AssemblyName><NeoCLRRoot>{escape(str(root))}</NeoCLRRoot></PropertyGroup>
<Import Project="{escape(str(ROOT / 'build/NeoCLR.Raven.props'))}" />
<ItemGroup><Compile Include="Main.rvn" /></ItemGroup></Project>''')
        diagnostic = run(['dotnet', compiler, project, '--no-project-restore', '-o', folder / 'compiled'], success)
        if not success:
            return diagnostic
        return folder / 'compiled' / (name + '.dll')

    image = compile('GenericInstance', source)
    imported = root / 'imported'
    run(['dotnet', bridge, '--library-implementation', image, core, 'Probe.Cell', imported])
    body = (imported / 'Implementation.neoil').read_text()
    if args.private_methods:
        assert '.method private instance Store(' in body
        assert '.method private instance Read(' in body
        bad = compile('PublicHelper', source.replace('private func Store', 'public func Store'))
        diagnostic = run(['dotnet', bridge, '--library-implementation', bad, core,
                          'Probe.Cell', root / 'public-helper-import'], False)
        assert 'does not match reference contract' in diagnostic, diagnostic

    driver = '.module GenericInstance\n.entry Main\n' + body + '\n.function Main() -> void\n'
    for value_type in ('Int32', 'String', 'Void'):
        driver += f'.local Probe.Cell<{value_type}> cell{value_type}\n.local System.Collections.Iterator<{value_type}> iterator{value_type}\n'
    for value_type, first, second in [('Int32', 'ldc.i4 7', 'ldc.i4 42'),
                                       ('String', 'ldstr "first"', 'ldstr "second"'),
                                       ('Void', 'ldvoid', 'ldvoid')]:
        owner = f'Probe.Cell<{value_type}>'
        iterator = f'System.Collections.Iterator<{value_type}>'
        driver += f'''{first}
newobj instance {owner}::.ctor({value_type})
stloc cell{value_type}
ldloc cell{value_type}
call instance {owner}::Self()
{second}
call instance {owner}::Set({value_type})
ldloc cell{value_type}
{second}
newobj instance {owner}::.ctor({value_type})
call instance {owner}::CopyFrom({owner})
ldloc cell{value_type}
call instance {owner}::Copy()
{first}
call instance {owner}::Set({value_type})
ldloc cell{value_type}
call instance {owner}::AsIterable()
callvirt instance System.Collections.Iterable<{value_type}>::GetIterator()
stloc iterator{value_type}
ldloc iterator{value_type}
callvirt instance {iterator}::MoveNext()
brfalse Failed
ldloc iterator{value_type}
callvirt instance {iterator}::get_Current()
'''
        driver += 'pop\nldstr "void"\n' if value_type == 'Void' else ''
        driver += f'call System.Console::WriteLine({"String" if value_type == "Void" else value_type})\npop\n'
        driver += f'ldloc iterator{value_type}\ncallvirt instance {iterator}::MoveNext()\nbrtrue Failed\nldloc iterator{value_type}\ncallvirt instance System.Disposable::Dispose()\n'
    driver += 'ret\nFailed:\nfault "Unexpected iterator cardinality"\n.end\n'
    application = root / 'App.neoil'
    application.write_text(driver)
    system = root / 'System.neoil'
    system.write_text(build(ROOT / 'runtime/System.neoil'))
    run([runtime, 'verify', application, '--system', system])
    assert run([runtime, 'run', application, '--system', system]).splitlines() == ['42', 'second', 'void']
    if args.private_methods:
        application.write_text(driver.replace('::Set(', '::Store('))
        diagnostic = run([runtime, 'verify', application, '--system', system], False)
        assert 'private' in diagnostic.lower() or 'accessible' in diagnostic.lower(), diagnostic
        application.write_text(driver)
    for name, invalid in [('Arity', source.replace('Cell<T>', 'Cell<T,U>')),
                          ('MissingInterface', source.replace(' : Iterable<T>', '').replace('public func AsIterable() -> Iterable<T> {\n        return self', 'public func AsIterable() -> Iterable<T> {\n        return ArrayList<T>()'))]:
        bad = compile(name, invalid)
        output = root / name / 'imported'
        diagnostic = run(['dotnet', bridge, '--library-implementation', bad, core, 'Probe.Cell', output], False)
        assert 'match reference contract' in diagnostic, diagnostic
        assert not (output / 'Implementation.neoil').exists()
    if args.checked_storage or args.private_storage:
        slot = 'items' if args.private_storage else 'stored'
        for name, invalid, expected in [
            ('Unreadable', source.replace(slot + '[0] = value', '', 1), 'uninitialized'),
            ('ZeroCapacity', source.replace('Reserve<T>(1)', 'Reserve<T>(0)'), 'out of range'),
            ('NegativeCapacity', source.replace('Reserve<T>(1)', 'Reserve<T>(-1)'), 'non-negative'),
        ]:
            bad = compile(name, invalid)
            output = root / name / 'imported'
            run(['dotnet', bridge, '--library-implementation', bad, core, 'Probe.Cell', output])
            app = root / name / 'App.neoil'
            app.write_text('.module StorageFailure\n.entry Main\n' + (output / 'Implementation.neoil').read_text() +
                '\n.function Main() -> void\nldc.i4 42\nnewobj instance Probe.Cell<Int32>::.ctor(Int32)\ncall instance Probe.Cell<Int32>::get_Value()\npop\nret\n.end\n')
            diagnostic = run([runtime, 'run', app, '--system', system], False)
            assert expected in diagnostic, diagnostic
        print('Checked storage rejects unwritten reads, zero-capacity stores and negative capacity')
    if args.private_storage:
        for name, invalid, expected in [
            ('PublicDependency', source.replace('internal class Storage', 'public class Storage'), 'Unsupported application'),
            ('PublicStorage', source.replace('private field items', 'public field items'), 'Unsupported private library dependency'),
        ]:
            bad = compile(name, invalid)
            output = root / name / 'imported'
            diagnostic = run(['dotnet', bridge, '--library-implementation', bad, core, 'Probe.Cell', output], False)
            assert expected in diagnostic, diagnostic
            assert not (output / 'Implementation.neoil').exists()
        print('Public helper identities and exposed implementation state are rejected')
    if args.checked_storage or args.private_storage:
        saved = core.read_bytes()
        try:
            run(['dotnet', bridge, '--reference-core', core])
            diagnostic = compile('ConsumerCore', source, False)
            assert 'error RAV0234' in diagnostic, diagnostic
        finally:
            core.write_bytes(saved)
        print('The ordinary consumer core does not expose checked storage')
    print('Generic instance fields, constructors, self signatures, Iterable dispatch and Int32/String/Void payloads passed; arity/interface mismatches rejected')
