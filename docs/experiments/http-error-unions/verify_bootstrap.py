#!/usr/bin/env python3
"""Import an empty-case Raven union against a separate core reference contract."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--bundle', required=True, type=Path)
parser.add_argument('--bridge', required=True, type=Path)
parser.add_argument('--runner', required=True, type=Path)
args = parser.parse_args()
bundle = args.bundle.resolve()
bridge = args.bridge.resolve()
env = dict(os.environ, NeoCLRRoot=str(bundle), RavenSdkRoot=str(bundle / 'raven-sdk'))


def run(command, success=True):
    result = subprocess.run([str(part) for part in command], env=env,
                            capture_output=True, text=True, timeout=180)
    if success and result.returncode:
        raise AssertionError(result.stdout + result.stderr)
    return result


with tempfile.TemporaryDirectory(prefix='neoclr-union-bootstrap-') as folder:
    root = Path(folder)
    (root / 'Limit.rvn').write_text('''namespace Probe
import System.*
public union Limit {
    case Headers
    case Body

    val IsHeader: bool {
        get {
            if let Limit.Headers = self {
                return true
            }
            return false
        }
    }

    override func ToString() -> string {
        if let Limit.Headers = self {
            return "Headers"
        }
        if let Limit.Body = self {
            return "Body"
        }
        return "Empty"
    }
}
''')
    (root / 'Limit.rvnproj').write_text('''<Project>
  <Import Project="$(NeoCLRRoot)/build/NeoCLR.Raven.props" />
  <PropertyGroup><OutputType>Library</OutputType></PropertyGroup>
  <ItemGroup><Compile Include="Limit.rvn" /></ItemGroup>
</Project>''')
    run(['dotnet', bundle / 'raven-sdk/tools/rvnc/rvnc.dll', root / 'Limit.rvnproj',
         '--no-project-restore', '-o', root / 'compiled'])
    source = root / 'compiled/Limit.dll'
    reference = root / 'NeoCLR.CoreProbe.dll'
    run(['dotnet', bridge, '--standard-union-library-core', source,
         bundle / 'demo/NeoCLR.CoreProbe.dll', 'Probe.Limit', reference])
    source_metadata = run(['dotnet', bridge, '--union-metadata-report', source]).stdout
    reference_metadata = run(['dotnet', bridge, '--union-metadata-report', reference]).stdout
    def limit_metadata(text):
        return next(item for line in text.splitlines() if (item := json.loads(line))['Carrier'] == 'Probe.Limit')
    assert limit_metadata(source_metadata) == limit_metadata(reference_metadata)
    (root / 'Consumer.rvn').write_text("""import Probe.*
func Create() -> Limit {
    Limit.Headers
}
func Inspect(value: Limit) -> bool {
    if let Limit.Headers = value {
        return true
    }
    return false
}
""")
    (root / 'Consumer.rvnproj').write_text(f'''<Project>
<Import Project="$(NeoCLRRoot)/build/NeoCLR.Raven.props" />
<PropertyGroup><OutputType>Library</OutputType></PropertyGroup>
<ItemGroup>
<Reference Update="NeoCLR.CoreProbe"><HintPath>{reference}</HintPath></Reference>
<Compile Include="Consumer.rvn" />
</ItemGroup>
</Project>''')
    run(['dotnet', bundle / 'raven-sdk/tools/rvnc/rvnc.dll', root / 'Consumer.rvnproj',
         '--no-project-restore', '-o', root / 'consumer'])
    print('Separate Raven consumer compiled against projected core union metadata.')
    # Recompile the union with the shared protocol now owned by the core reference.
    project = root / 'Limit.rvnproj'
    project.write_text(project.read_text().replace('<Compile Include="Limit.rvn" />',
        f'<Reference Update="NeoCLR.CoreProbe"><HintPath>{reference}</HintPath></Reference>'
        '<Compile Include="Limit.rvn" />'))
    run(['dotnet', bundle / 'raven-sdk/tools/rvnc/rvnc.dll', project,
         '--no-project-restore', '-o', root / 'shared'])
    shared_source = root / 'shared/Limit.dll'
    shared_import = root / 'shared-import'
    run(['dotnet', bridge, '--library-implementation', shared_source, reference,
         'Probe.Limit', shared_import])
    shared_body = (shared_import / 'Implementation.neoil').read_text()
    assert '.implements System.Runtime.CompilerServices.IUnion' in shared_body
    assert '.interface System.Runtime.CompilerServices.IUnion' not in shared_body

    imported = root / 'imported'
    run(['dotnet', bridge, '--library-implementation', source, reference, 'Probe.Limit', imported])
    body = (imported / 'Implementation.neoil').read_text()
    assert 'Raven.Runtime.CompilerServices' not in body, body
    assert 'out(true)' in body, body
    assert '.type Probe.Limit' in body and 'Application.V1_' not in body, body
    program = root / 'App.neoil'
    program.write_text('''.module UnionBootstrap
.entry Main
''' + body + '''
.function Main() -> void
.local Probe.Limit limit
.local Probe.Limit.Headers headers
.local Probe.Limit.Body body
.local System.Object boxed
ldloca headers
initobj Probe.Limit.Headers
ldloc headers
newobj instance Probe.Limit::.ctor(Probe.Limit.Headers)
stloc limit
ldloc limit
box Probe.Limit
stloc boxed
ldloca limit
call instance Probe.Limit::get_Value()
pop
ldloca limit
call instance Probe.Limit::get_IsHeader()
brfalse WrongHeaderProperty
ldloca limit
ldloca body
call instance Probe.Limit::TryGetValue(Probe.Limit.Body&)
brtrue WrongBodyExtraction
ldloca limit
ldloca headers
call instance Probe.Limit::TryGetValue(Probe.Limit.Headers&)
brfalse MissingHeaderExtraction
ldloca limit
call instance Probe.Limit::ToString()
call System.Console::WriteLine(String)
pop
ldloca body
initobj Probe.Limit.Body
ldloc body
newobj instance Probe.Limit::.ctor(Probe.Limit.Body)
stloc limit
ldloca limit
call instance Probe.Limit::get_IsHeader()
brtrue WrongBodyProperty
ldloc boxed
callvirt instance System.Object::ToString()
call System.Console::WriteLine(String)
pop
ldloca limit
call instance Probe.Limit::get_Value()
pop
ldloca limit
call instance Probe.Limit::ToString()
call System.Console::WriteLine(String)
pop
ldloca limit
initobj Probe.Limit
ldloca limit
call instance Probe.Limit::get_HasValue()
brtrue WrongDefault
ldloca limit
call instance Probe.Limit::ToString()
call System.Console::WriteLine(String)
pop
ret
WrongHeaderProperty:
fault "WrongHeaderProperty"
WrongBodyExtraction:
fault "WrongBodyExtraction"
MissingHeaderExtraction:
fault "MissingHeaderExtraction"
WrongBodyProperty:
fault "WrongBodyProperty"
WrongDefault:
fault "WrongDefault"
.end
''')
    system = bundle / 'lib/System.neoil'
    verified = run([bundle / 'bin/neoclr', 'verify', program, '--system', system], success=False)
    assert verified.returncode == 0, verified.stdout + verified.stderr + program.read_text()
    result = run([args.runner.resolve(), program, system, '64', '10000000'])
    assert result.stdout.splitlines() == ['Headers', 'Headers', 'Body', 'Empty'], result.stdout
    assert 'live=0' in result.stderr, result.stderr
    print(result.stdout + result.stderr)
    protocol = '' if '.interface System.Runtime.CompilerServices.IUnion\n' in system.read_text() else (
        '.interface System.Runtime.CompilerServices.IUnion\n'
        '.method instance get_Value() -> System.Object\n.end\n.end\n')
    shared_program = root / 'Shared.neoil'
    shared_program.write_text(program.read_text().replace(body, protocol + shared_body))
    shared_result = run([args.runner.resolve(), shared_program, system, '64', '10000000'])
    assert shared_result.stdout == result.stdout, shared_result.stdout
    assert 'live=0' in shared_result.stderr, shared_result.stderr
    print('Core-owned protocol: ' + shared_result.stderr)
    (root / 'Main.rvn').write_text("""import System.*
import System.Runtime.CompilerServices.*
import Probe.*
func Main() {
    let value: Limit = Limit.Headers
    let boxed: Object = value
    let view = (IUnion)boxed
    if view.Value is Limit.Headers {
        Console.WriteLine("Shared union case")
    } else {
        System.Fault("Wrong shared union case")
    }
}
""")
    (root / 'Application.rvnproj').write_text(f'''<Project>
<Import Project="$(NeoCLRRoot)/build/NeoCLR.Raven.props" />
<ItemGroup>
<Reference Update="NeoCLR.CoreProbe"><HintPath>{reference}</HintPath></Reference>
<Compile Include="Main.rvn" /><Compile Include="Limit.rvn" />
</ItemGroup>
</Project>''')
    run(['dotnet', bundle / 'raven-sdk/tools/rvnc/rvnc.dll', root / 'Application.rvnproj',
         '--no-project-restore', '-o', root / 'application'])
    run(['dotnet', bridge, '--import', root / 'application/Application.dll', reference,
         root / 'application-import'])
    application_system = root / 'System.neoil'
    application_system.write_text(system.read_text() + '\n' + protocol)
    application_result = run([args.runner.resolve(), root / 'application-import/App.neoil',
        application_system, '64', '10000000'])
    assert 'live=0' in application_result.stderr, application_result.stderr
    assert application_result.stdout.splitlines() == ['Shared union case'], application_result.stdout
    print('Shared protocol application dispatch: ' + application_result.stdout + application_result.stderr)

    for mutation in ('wrong-case', 'wrong-return', 'wrong-output', 'nonempty-case', 'unmarked',
                     'missing-case-metadata', 'wrong-case-ordinal', 'wrong-case-name', 'wrong-protocol'):
        altered = root / (mutation + '.dll')
        run(['dotnet', bridge, '--standard-union-library-core', source,
             bundle / 'demo/NeoCLR.CoreProbe.dll', 'Probe.Limit', altered, mutation])
        destination = root / mutation
        rejected = run(['dotnet', bridge, '--library-implementation', source, altered,
                        'Probe.Limit', destination], success=False)
        assert rejected.returncode != 0, 'Unexpected contract admission: ' + mutation
        assert ('standard union' in rejected.stdout + rejected.stderr
                or 'Standard union' in rejected.stdout + rejected.stderr
                or 'Raven union' in rejected.stdout + rejected.stderr
                or mutation == 'wrong-protocol' and 'unresolved member System.Object System.Runtime.CompilerServices.IUnion::get_Value()'
                in rejected.stdout + rejected.stderr), rejected.stdout + rejected.stderr
        assert not (destination / 'Implementation.neoil').exists()
        print('Rejected mismatched bootstrap contract: ' + mutation)
