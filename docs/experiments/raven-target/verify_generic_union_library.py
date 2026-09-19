"""Check generic union layout, constructors and conditional output admission."""
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


with tempfile.TemporaryDirectory(prefix='neoclr-foundation-library-') as temporary:
    root = Path(temporary)
    (root / 'demo').mkdir()
    core = root / 'demo/NeoCLR.CoreProbe.dll'
    run(['dotnet', args.bridge.resolve(), '--reference-library-core', core])
    option = (ROOT / 'runtime/raven/src/System/Option.rvn').read_text()
    result = (ROOT / 'runtime/raven/src/System/Result.rvn').read_text()
    cases = [
        ('Option', 'Option', option, None),
        ('Result', 'Result', result, None),
        ('NoneConstructorEffect', 'Option', option.replace('struct None { }', 'struct None { init() { System.Fault("unexpected") } }'), 'empty constructor'),
        ('ExtraCaseStorage', 'Option', option.replace('private field Stored: T', 'private field Extra: int\n        private field Stored: T'), 'union storage layout'),
        ('RenamedCase', 'Option', option.replace('Some', 'Different'), 'union case container'),
        ('ReferenceCase', 'Option', option.replace('struct Some<T>', 'class Some<T>'), 'value/reference representation'),
        ('CaseConstructorEffect', 'Option', option.replace('Stored = value', 'System.Fault("unexpected")\n            Stored = value'), 'union case constructor'),
        ('CarrierConstructorEffect', 'Option', option.replace('Stored = ValueStorage.Pack(value)', 'System.Fault("unexpected")\n        Stored = ValueStorage.Pack(value)', 1), 'carrier constructor'),
        ('WrongPayload', 'Option', option.replace('ValueStorage.Is<System.Option.Some<T>>', 'ValueStorage.Is<T>'), 'Unsupported case storage payload'),
        ('InvalidDefault', 'Option', option.replace('return System.Option<T>(System.Option.None())', 'return default(System.Option<T>)'), 'Read of uninitialized'),
        ('UnassignedSuccess', 'Option', option.replace('ValueStorage.LeaveUnassigned(out value)\n        return false', 'ValueStorage.LeaveUnassigned(out value)\n        return true'), 'must return false immediately'),
        ('WrongParameterName', 'Result', result.replace('out output: T', 'out destination: T').replace('output =', 'destination =').replace('out output)', 'out destination)'), 'export does not match'),
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
            assert '.field private Stored Value' in emitted
            assert '.field Value T0' in emitted
            assert 'readonly byref TryGetOutput(out(true) T0& destination)' in emitted
            assert 'ldc.bool false' in emitted
            assert 'starg this' in emitted
    print(f'{len(cases)} generic union admission cases passed')
