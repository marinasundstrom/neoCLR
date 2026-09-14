"""Compile independent Raven libraries and a consumer, then audit/import/execute."""
import argparse
import json
from pathlib import Path
import shutil
import subprocess
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('project', type=Path)
for name in ('compiler', 'bridge', 'runtime', 'system'):
    parser.add_argument('--' + name, type=Path, required=True)
args = parser.parse_args()
project_text = args.project.resolve().read_text()
results = {}

def run(command):
    return subprocess.run([str(x) for x in command], text=True, capture_output=True, timeout=120)

def require(result, label):
    assert result.returncode == 0, label + ': ' + result.stdout + result.stderr

with tempfile.TemporaryDirectory(prefix='neoclr-library-import-') as temporary:
    root = Path(temporary)
    core = root / 'NeoCLR.CoreProbe.dll'
    shutil.copyfile(args.project.resolve().parent / core.name, core)

    def compile(name, source, references=(), library=False, identity=None):
        identity = identity or name
        folder = root / name
        folder.mkdir()
        shutil.copyfile(core, folder / core.name)
        text = project_text.replace('<OutputType>Exe</OutputType>', '<OutputType>Library</OutputType>' if library else '<OutputType>Exe</OutputType>')
        text = text.replace('<PropertyGroup>', '<PropertyGroup>\n<AssemblyName>' + identity + '</AssemblyName>', 1)
        for path in references:
            shutil.copyfile(path, folder / path.name)
            text = text.replace('</ItemGroup>', f'<Reference Include="{path.stem}"><HintPath>{path.name}</HintPath></Reference></ItemGroup>', 1)
        (folder / 'Demo.rvnproj').write_text(text)
        (folder / 'Main.rvn').write_text(source)
        result = run(['dotnet', args.compiler.resolve(), folder / 'Demo.rvnproj', '--framework', 'net11.0', '--no-project-restore', '-o', folder / 'compiled'])
        require(result, name)
        return folder / 'compiled' / (identity + '.dll')

    arithmetic_source = '''public class Arithmetic {
    public static func Sum(value: int) -> int {
        if value == 0 {
            return 0
        }
        return value + Sum(value - 1)
    }
}
'''
    arithmetic = compile('ArithmeticLibrary', arithmetic_source, library=True)
    facade = compile('FacadeLibrary', '''namespace Workflows

public func Answer() -> int {
    return Arithmetic.Sum(8) + 6
}

public class Facade {
    public static func Forward() -> int {
        return Answer()
    }
}
''', [arithmetic], library=True)
    consumer = compile('Consumer', '''import System.Console.*
import Workflows.*
func Main() {
    WriteLine(Answer())
    WriteLine(Arithmetic.Sum(3))
}
''', [arithmetic, facade])

    def import_case(name, libraries, success=True, diagnostic=None):
        output = root / name
        result = run(['dotnet', args.bridge.resolve(), '--import', consumer, core, output, *libraries])
        if not success:
            assert result.returncode != 0 and not (output / 'App.neoil').exists(), result.stdout + result.stderr
            if diagnostic:
                assert diagnostic in result.stdout + result.stderr, result.stdout + result.stderr
            results[name] = 'rejected without executable output'
            return
        require(result, name)
        require(run([args.runtime.resolve(), 'verify', output / 'App.neoil', '--system', args.system.resolve()]), name)
        executed = run([args.runtime.resolve(), 'run', output / 'App.neoil', '--system', args.system.resolve()])
        require(executed, name)
        assert executed.stdout == '42\n6\n', executed.stdout
        mapping = json.loads((output / 'App.neoil.map.json').read_text())
        identities = mapping['MethodIdentities']
        assert len({m['AssemblyIdentity'] for m in identities}) == 3
        # Distinct assemblies normally reuse the same MethodDef rows. Both must run.
        assert len({m['MethodToken'] for m in identities}) < len(identities)
        lines = (output / 'App.neoil').read_text().splitlines()
        assert all(lines[m['OutputLine'] - 1] == m['OutputLabel'] for m in mapping['Mappings'])
        results[name] = 'separately compiled, imported, verified and executed; qualified source maps checked'

    import_case('transitive-libraries', [arithmetic, facade])
    import_case('reverse-input-order', [facade, arithmetic])
    import_case('missing-library', [facade], False, 'unresolved')
    import_case('duplicate-library', [arithmetic, facade, arithmetic], False, 'Duplicate assembly identity')
    private = compile('PrivateImplementation', arithmetic_source.replace('public static func Sum', 'private static func Sum'), library=True, identity='ArithmeticLibrary')
    import_case('private-implementation', [private, facade], False, 'Nonpublic cross-type call')
    changed = compile('ChangedSignature', arithmetic_source.replace('Sum(value: int)', 'Sum(value: int, unused: int)').replace('Sum(value - 1)', 'Sum(value - 1, 0)'), library=True, identity='ArithmeticLibrary')
    import_case('changed-signature', [changed, facade], False, 'unresolved')
    unsupported = compile('UnsupportedBody', arithmetic_source.replace('return value + Sum(value - 1)', 'return Counter().Read()') + '\npublic class Counter { public func Read() -> int { return 42 } }\n', library=True, identity='ArithmeticLibrary')
    import_case('library-instance-body', [unsupported, facade], False)

print(json.dumps(results, indent=2))
