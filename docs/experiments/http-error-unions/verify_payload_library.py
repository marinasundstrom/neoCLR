#!/usr/bin/env python3
"""Project/import a nongeneric data-bearing HTTP error library prototype."""
import argparse
import os
from pathlib import Path
import re
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
owner = 'System.Web.Http.HttpError'


def run(command, success=True):
    result = subprocess.run([str(x) for x in command], env=env, capture_output=True,
                            text=True, timeout=180)
    if success:
        assert result.returncode == 0, result.stdout + result.stderr
    return result


with tempfile.TemporaryDirectory(prefix='neoclr-http-payload-library-') as directory:
    root = Path(directory)
    original = Path(__file__).with_name('PayloadError.rvn').read_text()
    (root / 'Errors.rvn').write_text(original)
    core = bundle / 'demo/NeoCLR.CoreProbe.dll'

    def compile(name, source, reference):
        project = root / (name + '.rvnproj')
        project.write_text(f'''<Project>
<Import Project="$(NeoCLRRoot)/build/NeoCLR.Raven.props" />
<PropertyGroup><OutputType>Library</OutputType></PropertyGroup>
<ItemGroup>
<Reference Update="NeoCLR.CoreProbe"><HintPath>{reference}</HintPath></Reference>
<Compile Include="{source}" />
</ItemGroup>
</Project>''')
        output = root / name
        run(['dotnet', bundle / 'raven-sdk/tools/rvnc/rvnc.dll', project,
             '--no-project-restore', '-o', output])
        return output / (name + '.dll')

    source = compile('Source', 'Errors.rvn', core)
    projected = root / 'NeoCLR.CoreProbe.dll'
    run(['dotnet', bridge, '--project-union-reference', source, core, owner, projected])
    (root / 'Consumer.rvn').write_text('''import System.*
import System.Web.Http.*
import System.Networking.Sockets.*
func Create() -> HttpError { HttpError.Transport(SocketError.Closed) }
func Inspect(error: HttpError) -> bool {
    if let HttpError.Transport(SocketError.Closed) = error { return true }
    return false
}
func Message() -> HttpError { HttpError.Protocol("bad status") }
''')
    compile('Consumer', 'Consumer.rvn', projected)
    shared = compile('Shared', 'Errors.rvn', projected)
    imported = root / 'imported'
    run(['dotnet', bridge, '--library-implementation', shared, projected, owner, imported])
    body = (imported / 'Implementation.neoil').read_text()
    assert 'Raven.Runtime.CompilerServices' not in body
    assert '.interface System.Runtime.CompilerServices.IUnion' not in body
    assert 'out(true)' in body
    checks = []
    for variant, family, case in [
        ('InvalidUri', 'System.UriError', 'InvalidFormat'),
        ('NameResolution', 'System.Networking.DnsError', 'NoAddress'),
        ('Transport', 'System.Networking.Sockets.SocketError', 'ConnectionRefused'),
    ]:
        checks.append(f'''newobj instance {family}.{case}::.ctor()
newobj instance {family}::.ctor({family}.{case})
newobj instance {owner}.{variant}::.ctor({family})
newobj instance {owner}::.ctor({owner}.{variant})
stloc error
ldloca error
call instance {owner}::ToString()
call System.Console::WriteLine(String)
pop
''')
    program = root / 'App.neoil'
    program.write_text('.module HttpPayloadLibrary\n.entry Main\n' + body + f'''
.function Main() -> Void
.local {owner} error
.local {owner} copy
.local {owner}.Protocol protocol
.local {owner}.Transport transport
.local System.Object boxed
.local Int32 index
''' + ''.join(checks) + f'''
ldstr "retained message"
newobj instance {owner}.Protocol::.ctor(String)
newobj instance {owner}::.ctor({owner}.Protocol)
stloc error
ldloc error
stloc copy
ldloc error
box {owner}
stloc boxed
ldloca error
initobj {owner}
ldloca error
call instance {owner}::get_HasValue()
brtrue WrongDefault
ldloca copy
ldloca transport
call instance {owner}::TryGetValue({owner}.Transport&)
brtrue WrongCase
ldloca copy
ldloca protocol
call instance {owner}::TryGetValue({owner}.Protocol&)
brfalse MissingCase
ldc.i4 0
stloc index
Again:
ldloc copy
box {owner}
pop
ldloc index
ldc.i4 1
add
stloc index
ldloc index
ldc.i4 100
blt Again
ldloca protocol
call instance {owner}.Protocol::get_Message()
call System.Console::WriteLine(String)
pop
ldloc boxed
callvirt instance System.Object::ToString()
call System.Console::WriteLine(String)
pop
ldloca error
call instance {owner}::ToString()
call System.Console::WriteLine(String)
pop
ldvoid
ret
WrongDefault:
fault "Default invented an HTTP error"
WrongCase:
fault "Wrong HTTP variant extracted"
MissingCase:
fault "HTTP message payload lost"
.end
''')
    result = run([args.runner.resolve(), program, bundle / 'lib/System.neoil', '32', '1000000'])
    assert result.stdout.splitlines() == ['InvalidFormat', 'NoAddress', 'ConnectionRefused',
                                          'retained message', 'retained message', 'Empty'], result.stdout
    assert 'live=0' in result.stderr and re.search(r'collections=[1-9]', result.stderr), result.stderr
    print('Projected consumer, imported payloads, cases, defaults, copies and boxing: passed')
    print(result.stderr, end='')

    private_name = re.search(r'\.method private static (Metadata_\w+)\(System.Object value\)', body)[1]
    denied = root / 'Denied.neoil'
    denied.write_text('.module Denied\n.entry Main\n' + body + f'\n.function Main() -> String\nldstr "hidden"\ncastclass System.Object\ncall {owner}.Protocol::{private_name}(System.Object)\nret\n.end\n')
    result = run([args.runner.resolve(), denied, bundle / 'lib/System.neoil', '32', '1000000'], success=False)
    assert result.returncode != 0 and 'method access denied' in result.stderr, result.stdout + result.stderr
    print('Private generated formatter remains inaccessible to callers')

    # Keep nonempty overlapping layouts out; changing payload contracts must also fail.
    (root / 'Errors.rvn').write_text(original.replace('case Protocol(message: string)', 'case Protocol(message: string, detail: string)').replace('Protocol(let message) => message', 'Protocol(let message, _) => message'))
    changed = compile('Changed', 'Errors.rvn', core)
    result = run(['dotnet', bridge, '--library-implementation', changed, projected, owner, root / 'changed-import'], success=False)
    assert result.returncode != 0 and 'does not match reference contract' in result.stderr, result.stdout + result.stderr
    print('Changed payload contract: rejected')
    (root / 'Errors.rvn').write_text(original.replace('case Protocol(message: string)', 'case Protocol(message: int)').replace('Protocol(let message) => message', 'Protocol(let message) => message.ToString()'))
    overlapping = compile('Overlapping', 'Errors.rvn', core)
    rejected = root / 'rejected.dll'
    result = run(['dotnet', bridge, '--project-union-reference', overlapping, core, owner, rejected], success=False)
    assert result.returncode != 0 and 'Only supported nongeneric standard unions' in result.stderr, result.stdout + result.stderr
    assert not rejected.exists()
    print('Overlapping payload layout: rejected without publishing reference')
