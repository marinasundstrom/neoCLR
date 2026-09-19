"""Check inherited reflection snapshot source admission and storage."""
import argparse
from pathlib import Path
import subprocess
import tempfile
from xml.sax.saxutils import escape
from build_runtime_library import ROOT

parser = argparse.ArgumentParser(description=__doc__)
for name in ('compiler', 'bridge'):
    parser.add_argument('--' + name, required=True, type=Path)
args = parser.parse_args()


def run(command, diagnostic=None):
    result = subprocess.run([str(x) for x in command], capture_output=True, text=True, timeout=120)
    output = result.stdout + result.stderr
    assert (result.returncode == 0) == (diagnostic is None), output
    if diagnostic:
        assert diagnostic in output, (command, output)
    return result


with tempfile.TemporaryDirectory(prefix='neoclr-descriptor-library-') as temporary:
    root = Path(temporary)
    (root / 'demo').mkdir()
    core = root / 'demo/NeoCLR.CoreProbe.dll'
    run(['dotnet', args.bridge.resolve(), '--reference-library-core', core])
    source = (ROOT / 'runtime/raven/src/System/Introspection/Descriptors.rvn').read_text()
    cases = [
        ('Descriptors', source, None),
        ('MissingPermittedCase', source.replace('RuntimeMemberInfo, FieldInfo', 'RuntimeMemberInfo'), 'Introspection sealed hierarchy'),
        ('OpenModel', source.replace('public sealed interface', 'public interface'), 'Introspection sealed hierarchy'),
        ('ExtraStorage', source.replace('private field StoredName:', 'private field Extra: int\n    private field StoredName:'), 'Descriptor storage'),
        ('RenamedStorage', source.replace('StoredDefinitionIndex', 'StoredIndex'), 'Descriptor storage'),
        ('WrongStorage', source.replace('StoredIsPublic: bool', 'StoredIsPublic: int').replace('StoredIsPublic = isPublic', 'StoredIsPublic = 1').replace('get => StoredIsPublic', 'get => StoredIsPublic == 1'), 'Descriptor storage'),
        ('PublicConstructor', source.replace('protected init', 'public init'), 'Descriptor storage'),
        ('PublicProvider', source.replace('internal class RuntimeFieldInfo', 'public class RuntimeFieldInfo'), 'Runtime descriptor providers must remain internal'),
        ('OpenDescriptor', source.replace('internal class RuntimeFieldInfo', 'internal open class RuntimeFieldInfo'), 'sealing'),
        ('RenamedExport', source.replace('val CanWrite:', 'val Writable:'), 'does not match reference contract'),
        ('WrongParameterName', source.replace('arg0', 'includePrivate'), 'does not match reference contract'),
        ('SnapshotDefault', source.replace('val count = StoredParameters.Length', 'val snapshot: ParameterSnapshot = default(ParameterSnapshot)\n        val count = snapshot.Length'), 'RAV1509'),
    ]
    for name, text, diagnostic in cases:
        folder = root / name
        folder.mkdir()
        (folder / 'Main.rvn').write_text(text)
        project = folder / 'Probe.rvnproj'
        project.write_text(f'''<Project>
  <PropertyGroup><OutputType>Library</OutputType><AssemblyName>{name}</AssemblyName><NeoCLRRoot>{escape(str(root))}</NeoCLRRoot></PropertyGroup>
  <Import Project="{escape(str(ROOT / 'build/NeoCLR.Raven.props'))}" />
  <PropertyGroup><RavenTypeOfAssemblyName/><RavenTypeOfInfoType/><RavenTypeOfContextType/></PropertyGroup>
  <ItemGroup><Compile Include="Main.rvn" /></ItemGroup>
</Project>''')
        compile_diagnostic = 'RAV0501' if name == 'PublicProvider' else diagnostic if name == 'SnapshotDefault' else None
        run(['dotnet', args.compiler.resolve(), project, '--no-project-restore', '-o', folder / 'bin'], compile_diagnostic)
        if compile_diagnostic:
            continue
        output = folder / 'imported'
        run(['dotnet', args.bridge.resolve(), '--library-implementation', folder / 'bin' / (name + '.dll'),
             core, 'System.Introspection.MemberInfo', output], diagnostic)
        if diagnostic:
            assert not (output / 'Implementation.neoil').exists()
        else:
            emitted = (output / 'Implementation.neoil').read_text()
            assert '.extends System.Introspection.RuntimeMemberInfo' in emitted
            assert '.field private Parameters System.Introspection.ParameterInfo[]' in emitted
            assert 'GetParameters() -> System.Collections.Sequence<System.Introspection.ParameterInfo>' in emitted
    print(f'{len(cases)} descriptor admission cases passed')
