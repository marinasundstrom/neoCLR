"""Execute the isolated interface-based introspection migration probe."""
import argparse
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
source = Path(__file__).resolve().parent / 'introspection-v1'


def run(command, success=True):
    result = subprocess.run([str(x) for x in command], capture_output=True,
                            text=True, timeout=120)
    assert (result.returncode == 0) == success, result.stdout + result.stderr
    return result


with tempfile.TemporaryDirectory(prefix='neoclr-introspection-v1-') as temporary:
    root = Path(temporary)
    (root / 'demo').mkdir()
    run([*bridge, '--reference-core', root / 'demo/NeoCLR.CoreProbe.dll'])
    system = root / 'System.neoil'
    system.write_text(build(ROOT / 'runtime/System.neoil'))
    project = root / 'Probe.rvnproj'
    for path in source.glob('*.rvn'):
        (root / path.name).write_text(path.read_text())
    project.write_text(f'''<Project>
  <PropertyGroup><NeoCLRRoot>{escape(str(root))}</NeoCLRRoot></PropertyGroup>
  <Import Project="{escape(str(ROOT / 'build/NeoCLR.Raven.props'))}" />
  <ItemGroup>
    <Compile Include="TypeInfo.rvn" />
    <Compile Include="MemberInfo.rvn" />
    <Compile Include="FieldInfo.rvn" />
    <Compile Include="RuntimeTypeInfo.rvn" />
    <Compile Include="RuntimeFieldInfo.rvn" />
    <Compile Include="Main.rvn" />
  </ItemGroup>
</Project>''')
    output = root / 'valid'
    run([*bridge, '--project', project, output])
    artifact = output / 'App.neoil'
    emitted = artifact.read_text()
    interfaces = re.findall(r'^\.interface (Application\.V1_T_[0-9A-F]+)$', emitted, re.M)
    assert len(interfaces) == 3, emitted
    type_info = next(name for name in interfaces if bytes.fromhex(name.split('V1_T_')[1]).decode().endswith('8:TypeInfo'))
    field_info = next(name for name in interfaces if bytes.fromhex(name.split('V1_T_')[1]).decode().endswith('9:FieldInfo'))
    member_info = next(name for name in interfaces if bytes.fromhex(name.split('V1_T_')[1]).decode().endswith('10:MemberInfo'))
    assert '.interface ' + field_info + '\n.implements ' + member_info in emitted, emitted
    assert '.method instance GetFields(System.Introspection.BindingFlags) -> System.Collections.Iterable<' + field_info + '>' in emitted, emitted
    # The structural reference is the new interface, never legacy System.Type.
    assert '.method instance get_DeclaringType() -> ' + type_info in emitted, emitted
    assert '.method instance get_Type() -> ' + type_info in emitted, emitted
    try:
        run([runtime, 'verify', artifact, '--system', system])
    except AssertionError as error:
        raise AssertionError(str(error) + '\n' + emitted) from error
    result = run([runtime, 'run', artifact, '--system', system])
    # Preserve the current runtime's qualified Name behavior; the probe does not
    # establish a new naming or type-equality policy.
    assert result.stdout.strip().splitlines() == [
        'System.Int32', 'System', 'System.String', 'System',
        'StoredDayNumber', 'System.Date', 'System', 'System.Int32', 'System',
        'Private instance field', '0', '0', '1'], result.stdout

    main = (source / 'Main.rvn').read_text()
    for name, before, operation, diagnostic in [
        ('NoInvocation', 'Describe(integer)', 'integer.Invoke()', 'Invoke'),
        ('NoLegacyInfo', 'Describe(integer)', 'integer.Info', 'Info'),
        ('NoFieldAccess', 'Describe(descriptor.Type)', 'descriptor.GetValue()', 'GetValue'),
    ]:
        (root / 'Main.rvn').write_text(main.replace(before, operation))
        rejected = run([*bridge, '--project', project, root / name], False)
        assert diagnostic in rejected.stdout + rejected.stderr
        assert not (root / name / 'App.neoil').exists()

    # Unsupported flags still fault at query time, rather than looking like an
    # empty result or being silently discarded by the new adapter.
    (root / 'Main.rvn').write_text(main.replace(
        'BindingFlags.NonPublic | BindingFlags.Instance', '(BindingFlags)1'))
    output = root / 'invalid-flags'
    run([*bridge, '--project', project, output])
    artifact = output / 'App.neoil'
    run([runtime, 'verify', artifact, '--system', system])
    rejected = run([runtime, 'run', artifact, '--system', system], False)
    assert 'unsupported BindingFlags bits' in rejected.stdout + rejected.stderr
print('Shared field queries execute with TypeInfo-valued structural references; filters and invalid flags retain runtime behavior.')
