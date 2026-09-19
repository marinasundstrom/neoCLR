"""Check the normal BindingFlags enum declaration contract."""
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


with tempfile.TemporaryDirectory(prefix='neoclr-flags-library-') as temporary:
    root = Path(temporary)
    (root / 'demo').mkdir()
    core = root / 'demo/NeoCLR.CoreProbe.dll'
    run(['dotnet', args.bridge.resolve(), '--reference-library-core', core])
    source = (ROOT / 'runtime/raven/src/System/Introspection/BindingFlags.rvn').read_text()
    cases = [
        ('BindingFlags', source, None),
        ('WrongValue', source.replace('Public = 16', 'Public = 64'), 'Unsupported BindingFlags enum metadata'),
        ('MissingMember', source.replace('    NonPublic = 32\n', ''), 'Unsupported BindingFlags enum metadata'),
        ('ExtraMember', source.replace('Default = 0', 'Extra = 64\n    Default = 0'), 'Unsupported BindingFlags enum metadata'),
        ('WrongUnderlyingType', source.replace('enum BindingFlags {', 'enum BindingFlags : byte {'), 'Unsupported BindingFlags enum metadata'),
        ('MissingFlags', source.replace('[System.Flags]', ''), 'Unsupported BindingFlags enum metadata'),
        ('WrongKind', 'namespace System.Introspection\npublic struct BindingFlags {}\n', 'Unsupported BindingFlags enum metadata'),
    ]
    for name, text, diagnostic in cases:
        folder = root / name
        folder.mkdir()
        (folder / 'Main.rvn').write_text(text)
        project = folder / 'Probe.rvnproj'
        project.write_text(f'''<Project>
  <PropertyGroup><OutputType>Library</OutputType><AssemblyName>{name}</AssemblyName><NeoCLRRoot>{escape(str(root))}</NeoCLRRoot></PropertyGroup>
  <Import Project="{escape(str(ROOT / 'build/NeoCLR.Raven.props'))}" />
  <ItemGroup><Compile Include="Main.rvn" /></ItemGroup>
</Project>''')
        compile_diagnostic = None
        run(['dotnet', args.compiler.resolve(), project, '--no-project-restore', '-o', folder / 'bin'], compile_diagnostic)
        if compile_diagnostic:
            continue
        output = folder / 'imported'
        run(['dotnet', args.bridge.resolve(), '--library-implementation', folder / 'bin' / (name + '.dll'),
             core, 'System.Introspection.BindingFlags', output], diagnostic)
        if diagnostic:
            assert not (output / 'Implementation.neoil').exists()
    print(f'{len(cases)} enum implementation admission cases passed')
