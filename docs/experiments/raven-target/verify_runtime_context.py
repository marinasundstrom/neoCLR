"""Execute Raven typeof through RuntimeContext, then check handle identity separately."""
import argparse
import json
from pathlib import Path
import re
import subprocess
import tempfile
from xml.sax.saxutils import escape
from collection_library import ROOT, build

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--bridge', required=True, type=Path)
parser.add_argument('--runtime', required=True, type=Path)
args = parser.parse_args()
bridge = ['dotnet', str(args.bridge.resolve())]
runtime = str(args.runtime.resolve())
source = Path(__file__).parent / 'introspection-v1'


def run(command):
    result = subprocess.run([str(x) for x in command], capture_output=True,
                            text=True, timeout=120)
    assert result.returncode == 0, result.stdout + result.stderr
    return result


with tempfile.TemporaryDirectory(prefix='neoclr-runtime-context-') as temporary:
    root = Path(temporary)
    (root / 'demo').mkdir()
    run([*bridge, '--reference-core', root / 'demo/NeoCLR.CoreProbe.dll'])
    system = root / 'System.neoil'
    system.write_text(build(ROOT / 'runtime/System.neoil'))
    files = ['TypeInfo', 'MemberInfo', 'FieldInfo', 'RuntimeTypeInfo', 'RuntimeFieldInfo', 'RuntimeContext', 'ContextSample']
    for name in files:
        text = (source / (name + '.rvn')).read_text()
        if name == 'RuntimeTypeInfo':
            # Test instrumentation only: do not add identity helpers to the API.
            text = text.replace('    val Name:', '''    func HasSameIdentity(other: RuntimeTypeInfo) -> bool {
        return runtimeType.Equals(other: other.runtimeType)
    }

    val Name:''')
        (root / (name + '.rvn')).write_text(text)
    # This source entry must run unchanged: no injected instructions for typeof.
    (root / 'Main.rvn').write_text('''import System.*
import System.Introspection.*
import System.Runtime.*
func Main() {
    let sample = ContextSample()
    sample.Run()
    let identity = IdentitySource().Resolve()
}

class IdentitySource {
    func Resolve() -> TypeInfo {
        return typeof(int)
    }
}
''')
    includes = '\n'.join(f'<Compile Include="{name}.rvn" />' for name in files + ['Main'])
    project = root / 'Probe.rvnproj'
    project.write_text(f'''<Project>
  <PropertyGroup><NeoCLRRoot>{escape(str(root))}</NeoCLRRoot></PropertyGroup>
  <Import Project="{escape(str(ROOT / 'build/NeoCLR.Raven.props'))}" />
  <PropertyGroup>
    <RavenTypeOfAssemblyName>Probe</RavenTypeOfAssemblyName>
    <RavenTypeOfInfoType>System.Introspection.TypeInfo</RavenTypeOfInfoType>
    <RavenTypeOfContextType>System.Runtime.RuntimeContext</RavenTypeOfContextType>
  </PropertyGroup>
  <ItemGroup>{includes}</ItemGroup>
</Project>''')
    output = root / 'compiled'
    run([*bridge, '--project', project, output])
    artifact = output / 'App.neoil'
    run([runtime, 'verify', artifact, '--system', system])
    result = run([runtime, 'run', artifact, '--system', system])
    assert result.stdout.strip().splitlines() == [
        'System.Date', 'System', 'StoredDayNumber', 'System.Int32', 'System.Date'], result.stdout
    print(result.stdout, end='')
    mapping = json.loads((output / 'App.neoil.map.json').read_text())
    names = {entry['MetadataName']: entry['RuntimeName'] for entry in mapping['TypeIdentities']}
    context = names['System.Runtime.RuntimeContext']
    implementation = names['System.Runtime.RuntimeTypeInfo']
    identity_source = names['IdentitySource']
    info = names['System.Introspection.TypeInfo']
    current = next(entry['RuntimeName'] for entry in mapping['MethodIdentities']
                   if 'RuntimeContext::get_Current()' in entry['MetadataName'])
    emitted = re.sub(r'^\.entry .+$', '.entry ContextProbe', artifact.read_text(), flags=re.M)
    assert f'GetTypeInfoFromHandle(System.RuntimeTypeHandle) -> {info}' in emitted

    def resolve(token):
        return f'''call {current}()
ldtoken {token}
callvirt instance {context}::GetTypeInfoFromHandle(System.RuntimeTypeHandle)
'''

    # Check equivalent acquisition through separate Current facades, and preserve
    # scalar/array distinctions. Equality inspection is test-only on the hidden
    # implementation; consumers still receive the TypeInfo interface.
    cases = [('Int32', 'Int32', True), ('Int32', 'String', False),
             ('arrayref<Int32>', 'arrayref<Int32>', True),
             ('arrayref<Int32>', 'arrayref<String>', False)]
    body = f'.function ContextProbe() -> noresult\n.local {implementation} left\n'
    # Compare compiler-generated typeof directly with explicit context acquisition.
    body += f'newobj instance {identity_source}::.ctor()\n'
    body += f'callvirt instance {identity_source}::Resolve()\ncastclass {implementation}\nstloc left\nldloc left\n'
    body += resolve('Int32') + f'castclass {implementation}\ncall instance {implementation}::HasSameIdentity({implementation})\n'
    body += 'brtrue TypeOfIdentity\nfault "typeof and context identity mismatch"\nTypeOfIdentity:\n'
    for left, right, equal in cases:
        body += resolve(left) + f'castclass {implementation}\nstloc left\nldloc left\n'
        body += resolve(right) + f'castclass {implementation}\ncall instance {implementation}::HasSameIdentity({implementation})\n'
        label = 'Identity' + str(len(body))
        body += f'br{"true" if equal else "false"} {label}\nfault "descriptor identity mismatch"\n{label}:\n'
    body += 'ret\n.end\n'
    artifact.write_text(emitted + '\n' + body)
    run([runtime, 'verify', artifact, '--system', system])
    result = run([runtime, 'run', artifact, '--system', system])
    assert not result.stdout.strip(), result.stdout
print('Raven typeof executes through RuntimeContext as TypeInfo; repeated handles agree and distinct types/arrays differ.')
