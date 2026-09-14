"""Validate a Raven class implementation against a separate CLI reference contract."""
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
    run(['dotnet', bridge, '--instance-library-core', core])
    source = '''namespace Probe
public class Counter {
    private var stored: int
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
    public val Value: int {
        get { return stored }
    }
}
'''

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
    assert '.property instance Value() -> Int32' in body
    application = root / 'App.neoil'
    application.write_text('''.module InstanceLibrary
.entry Main
''' + body + '''
.function Main() -> void
.local Probe.Counter original
.local Probe.Counter alias
ldc.i4 7
newobj instance Probe.Counter::.ctor(Int32)
stloc original
ldloc original
call instance Probe.Counter::Self()
stloc alias
ldloc alias
ldc.i4 35
call instance Probe.Counter::Add(Int32)
ldloc original
call instance Probe.Counter::get_Value()
call System.Console::WriteLine(Int32)
pop
ldloc original
ldc.i4 99
newobj instance Probe.Counter::.ctor(Int32)
call instance Probe.Counter::CopyFrom(Probe.Counter)
ldloc alias
call instance Probe.Counter::get_Value()
call System.Console::WriteLine(Int32)
pop
ret
.end
''')
    system = root / 'System.neoil'
    system.write_text(build(ROOT / 'runtime/System.neoil'))
    run([runtime, 'verify', application, '--system', system])
    assert run([runtime, 'run', application, '--system', system]).splitlines() == ['42', '99']
    for name, invalid, diagnostic in [
        ('WrongName', source.replace('amount', 'increment'), 'does not match reference contract'),
        ('MissingMember', source.replace('    public func Self() -> Counter {\n        return self\n    }\n', ''), 'does not match reference contract'),
        ('PublicState', source.replace('private var stored', 'public var stored'), 'does not match reference contract'),
        ('ExtraMember', source.replace('    private var stored', '    public func Extra() -> int { return 1 }\n    private var stored'), 'does not match reference contract'),
        ('WrongCategory', source.replace('public class Counter', 'public struct Counter'), 'Unsupported instance library owner'),
    ]:
        bad = compile(name, invalid)
        output = root / name / 'imported'
        message = run(['dotnet', bridge, '--library-implementation', bad, core, 'Probe.Counter', output], False)
        assert diagnostic in message, name + ": " + message
        assert not (output / 'Implementation.neoil').exists()
    print('Instance library: constructor, private state, shared identity, self signatures, property, mutation and five rejected contracts passed')
