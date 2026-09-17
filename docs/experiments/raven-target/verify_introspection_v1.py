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
    assert len(interfaces) == 2, emitted
    type_info = next(name for name in interfaces if bytes.fromhex(name.split('V1_T_')[1]).decode().endswith('8:TypeInfo'))
    # The structural reference is the new interface, never legacy System.Type.
    assert '.method instance get_DeclaringType() -> ' + type_info in emitted, emitted
    run([runtime, 'verify', artifact, '--system', system])
    result = run([runtime, 'run', artifact, '--system', system])
    # Preserve the current runtime's qualified Name behavior; the probe does not
    # establish a new naming or type-equality policy.
    assert result.stdout.strip().splitlines() == [
        'System.Int32', 'System', 'System.String', 'System',
        'StoredDayNumber', 'System.Date', 'System'], result.stdout

    main = (source / 'Main.rvn').read_text()
    for name, operation in [('NoInvocation', 'integer.Invoke()'),
                            ('NoLegacyInfo', 'integer.Info')]:
        (root / 'Main.rvn').write_text(main.replace('Describe(integer)', operation))
        rejected = run([*bridge, '--project', project, root / name], False)
        assert ('Invoke' if name == 'NoInvocation' else 'Info') in rejected.stdout + rejected.stderr
        assert not (root / name / 'App.neoil').exists()
print('Runtime-backed TypeInfo/MemberInfo consumers execute; invocation and legacy Info are absent from the new contracts.')
