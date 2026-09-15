"""Validate a Raven value implementation against a separate CLI reference contract."""
import argparse
import json
from pathlib import Path
import subprocess
import tempfile
from xml.sax.saxutils import escape
from build_runtime_library import ROOT
from collection_library import build

parser = argparse.ArgumentParser(description=__doc__)
for name in ('compiler', 'bridge', 'runtime'):
    parser.add_argument('--' + name, required=True, type=Path)
parser.add_argument("--wide", action="store_true")
args = parser.parse_args()
compiler, bridge, runtime = (getattr(args, n).resolve() for n in ('compiler', 'bridge', 'runtime'))


def run(command, success=True):
    result = subprocess.run([str(x) for x in command], capture_output=True, text=True, timeout=120)
    assert (result.returncode == 0) == success, result.stdout + result.stderr
    return result.stdout + result.stderr


with tempfile.TemporaryDirectory(prefix='neoclr-instance-library-') as temporary:
    root = Path(temporary)
    (root / 'demo').mkdir()
    core = root / 'demo/NeoCLR.CoreProbe.dll'
    run(['dotnet', bridge, '--wide-value-instance-library-core' if args.wide else '--value-instance-library-core', core])
    source = '''namespace Probe
public struct Counter {
    private field stored: int
    public init(value: int) {
        stored = value
    }
    public func Add(amount: int) {
        stored += amount
    }
    public func CopyFrom(other: Counter) {
        stored = other.Value
    }
    public func Self() -> Counter {
        return self
    }
    public func Copy() -> Counter {
        return Counter(stored)
    }
    public val Value: int {
        get { return stored }
    }
}
'''

    if args.wide:
        source = source.replace(': int', ': long')

    def compile(name, text):
        folder = root / name
        folder.mkdir()
        (folder / 'Main.rvn').write_text(text)
        project = folder / 'Library.rvnproj'
        project.write_text(f'''<Project><PropertyGroup><OutputType>Library</OutputType>
<AssemblyName>{name}</AssemblyName><NeoCLRRoot>{escape(str(root))}</NeoCLRRoot></PropertyGroup>
<Import Project="{escape(str(ROOT / 'build/NeoCLR.Raven.props'))}" />
<ItemGroup><Compile Include="Main.rvn" /></ItemGroup></Project>''')
        run(['dotnet', compiler, project, '--no-project-restore', '-o', folder / 'compiled'])
        return folder / 'compiled' / (name + '.dll')

    image = compile('InstanceLibrary', source)
    imported = root / 'imported'
    run(['dotnet', bridge, '--library-implementation', image, core, 'Probe.Counter', imported])
    body = (imported / 'Implementation.neoil').read_text()
    mapping = json.loads((imported / 'Implementation.neoil.map.json').read_text())
    assert [t['RuntimeName'] for t in mapping['TypeIdentities']] == ['Probe.Counter']
    assert '.property instance Value() -> ' + ('Int64' if args.wide else 'Int32') in body
    application = root / 'App.neoil'
    application.write_text('''.module ValueLibrary
.entry Main
''' + body + '''
.function Main() -> void
.local Probe.Counter original
.local Probe.Counter copy
ldc.i4 7
newobj instance Probe.Counter::.ctor(Int32)
stloc original
ldloca original
call instance Probe.Counter::Self()
stloc copy
ldloca copy
call instance Probe.Counter::Copy()
stloc copy
ldloca copy
ldc.i4 35
call instance Probe.Counter::Add(Int32)
pop
ldloca original
call instance Probe.Counter::get_Value()
call System.Console::WriteLine(Int32)
pop
ldloca copy
call instance Probe.Counter::get_Value()
call System.Console::WriteLine(Int32)
pop
ldloca original
ldloc copy
call instance Probe.Counter::CopyFrom(Probe.Counter)
pop
ldloca original
call instance Probe.Counter::get_Value()
call System.Console::WriteLine(Int32)
pop
ret
.end
''')
    if args.wide:
        prefix, driver = application.read_text().split('.function Main()', 1)
        driver = driver.replace('Int32', 'Int64').replace('ldc.i4', 'ldc.i8')
        driver = driver.replace('ldc.i8 7\n', 'ldc.i8 4294967303\n')
        for index, (expected, printed) in enumerate([(4294967303, 7), (4294967338, 42), (4294967338, 42)]):
            driver = driver.replace('call System.Console::WriteLine(Int64)',
                f'ldc.i8 {expected}\nceq\nbrtrue Wide{index}\nfault "Wide value mismatch"\nWide{index}:\nldc.i4 {printed}\ncall System.Console::WriteLine(Int32)', 1)
        application.write_text(prefix + '.function Main()' + driver)
    assert any(m['RuntimeName'] == 'Probe.Counter::.ctor' for m in mapping['MethodIdentities'])
    system = root / 'System.neoil'
    system.write_text(build(ROOT / 'runtime/System.neoil'))
    run([runtime, 'verify', application, '--system', system])
    assert run([runtime, 'run', application, '--system', system]).splitlines() == ['7', '42', '42']
    original = application.read_text()
    prefix, driver = original.split('.function Main()', 1)
    application.write_text(prefix + '.function Main()' + driver.replace(
        'call instance Probe.Counter::get_Value()', 'ldfld Probe.Counter::stored'))
    denied = run([runtime, 'verify', application, '--system', system], False)
    assert 'field access denied' in denied, denied
    application.write_text(original)
    for name, invalid, diagnostic in [
        ('WrongName', source.replace('amount', 'increment'), 'does not match reference contract'),
        ('WrongFieldName', source.replace('stored', 'renamed'), 'value library layout'),
        ('ExtraField', source.replace('private field stored', 'private field extra: int\n    private field stored'), 'value library layout'),
        ('WrongCategory', source.replace('public struct Counter', 'public class Counter'), 'value/reference representation'),
    ]:
        bad = compile(name, invalid)
        output = root / name / 'imported'
        message = run(['dotnet', bridge, '--library-implementation', bad, core, 'Probe.Counter', output], False)
        assert diagnostic in message, name + ": " + message
        assert not (output / 'Implementation.neoil').exists()
    print('Value library: construction, independent copies, byref mutation, self signatures and layout/category rejection passed')
