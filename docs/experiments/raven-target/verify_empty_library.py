"""Validate payload-free error and nominal Void declarations against their reference contracts."""
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
        assert diagnostic in output, output
    return result


with tempfile.TemporaryDirectory(prefix='neoclr-foundation-library-') as temporary:
    root = Path(temporary)
    (root / 'demo').mkdir()
    core = root / 'demo/NeoCLR.CoreProbe.dll'
    run(['dotnet', args.bridge.resolve(), '--reference-library-core', core])
    names = ['InvalidRangeError', 'InvalidDateError', 'InvalidTimeError', 'OverflowError', 'EnvironmentError', 'Void']
    sources = {name: (ROOT / f'runtime/raven/src/System/{name}.rvn').read_text() for name in names}
    cases = [(name, name, sources[name], None) for name in names]
    cases += [
        ('Storage', 'InvalidRangeError', sources['InvalidRangeError'].replace('public struct InvalidRangeError {', 'public struct InvalidRangeError {\n    private field extra: int'), 'value library layout'),
        ('ExtraExport', 'EnvironmentError', sources['EnvironmentError'].replace('func ToString()', 'func Extra()'), 'export does not match'),
        ('VoidStorage', 'Void', sources['Void'].replace('public struct Void { }', 'public struct Void { private field extra: int }'), 'value library layout'),
    ]
    for name, owner, text, diagnostic in cases:
        folder = root / name
        folder.mkdir()
        (folder / 'Main.rvn').write_text(text)
        project = folder / 'Probe.rvnproj'
        project.write_text(f'''<Project>
  <PropertyGroup><OutputType>Library</OutputType><AssemblyName>{name}</AssemblyName><NeoCLRRoot>{escape(str(root))}</NeoCLRRoot></PropertyGroup>
  <Import Project="{escape(str(ROOT / 'build/NeoCLR.Raven.props'))}" />
  <ItemGroup><Compile Include="Main.rvn" /></ItemGroup>
</Project>''')
        run(['dotnet', args.compiler.resolve(), project, '--no-project-restore', '-o', folder / 'bin'])
        output = folder / 'imported'
        run(['dotnet', args.bridge.resolve(), '--library-implementation', folder / 'bin' / (name + '.dll'),
             core, 'System.' + owner, output], diagnostic)
        if diagnostic:
            assert not (output / 'Implementation.neoil').exists()
        else:
            emitted = (output / 'Implementation.neoil').read_text()
            assert '.type System.' + owner + '\n' in emitted
            assert '.field ' not in emitted
            if owner in ('Void', 'EnvironmentError'):
                assert '.method instance .ctor(' not in emitted
            elif diagnostic is None:
                assert '.method instance .ctor() -> Void' in emitted
                assert '.method instance ToString() -> String' in emitted
    print(f'{len(cases)} empty library admission cases passed')
