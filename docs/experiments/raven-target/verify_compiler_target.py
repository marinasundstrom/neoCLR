"""Compile through rvnc, then independently import and execute the resulting target PE."""
import argparse
import json
from pathlib import Path
import shutil
import subprocess
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('project', type=Path)
parser.add_argument('--compiler', type=Path, required=True)
parser.add_argument('--bridge', type=Path, required=True)
parser.add_argument('--system', type=Path, required=True)
parser.add_argument('--runtime', type=Path, required=True)
args = parser.parse_args()
root = Path(__file__).resolve().parent
project_text = args.project.resolve().read_text()
results = {}
with tempfile.TemporaryDirectory(prefix='neoclr-compiler-target-') as temporary:
    directory = Path(temporary)
    def compile_case(name, source, properties=None):
        case = directory / name
        case.mkdir()
        shutil.copyfile(args.project.resolve().parent / 'NeoCLR.CoreProbe.dll', case / 'NeoCLR.CoreProbe.dll')
        (case / 'Demo.rvnproj').write_text(project_text if properties is None else properties)
        (case / 'Main.rvn').write_text(source)
        run = subprocess.run(['dotnet', str(args.compiler.resolve()), str(case / 'Demo.rvnproj'),
                              '--framework', 'net11.0', '--no-project-restore', '-o', str(case / 'compiled')],
                             text=True, capture_output=True)
        return case, run

    cases = [
        ('normal-compiler', 'import System.Console.*\nfunc Main() { WriteLine("Normal Raven compiler") }\n',
         'Normal Raven compiler\n'),
        ('string-predicate', '''import System.*
import System.Collections.*
import System.Linq.*
import System.Console.*
func Main() {
    let orders: List<string> = ArrayList<string>()
    orders.Add("test")
    orders.Add("2")
    for item in orders.Filter(x => x == "2") {
        WriteLine(item)
    }
}
''', '2\n'),
        ('propagation', (root / 'samples/library-propagation-workflow.rvn').read_text(),
         '42\nSaved\nCompleted\nOverflow\nValue found\n42\nAbsent\n'),
    ]
    for name, source, expected in cases:
        case, compiled = compile_case(name, source)
        assert compiled.returncode == 0, name + ': ' + compiled.stdout + compiled.stderr
        imported = subprocess.run(['dotnet', str(args.bridge.resolve()), '--import',
                                   str(case / 'compiled/Demo.dll'), str(case / 'NeoCLR.CoreProbe.dll'),
                                   str(case / 'imported')], text=True, capture_output=True)
        assert imported.returncode == 0, name + ': ' + imported.stdout + imported.stderr
        mapping = json.loads((case / 'imported/App.neoil.map.json').read_text())
        assert mapping['IdentityEncoding'] == 'assembly-signature-v1'
        assert mapping['AssemblyIdentity'].startswith('Demo,')
        assert mapping['MethodIdentities'] and all(m['MetadataName'] and m['RuntimeName'] for m in mapping['MethodIdentities'])
        il = (case / 'imported/App.neoil').read_text()
        assert all(t['RuntimeName'] in il for t in mapping['TypeIdentities'])
        # Import audits the PE dependency closure against the supplied core. Leaked
        # host references therefore fail here rather than being silently replaced.
        command = [str(args.runtime.resolve()), 'verify', str(case / 'imported/App.neoil'),
                   '--system', str(args.system.resolve())]
        verified = subprocess.run(command, text=True, capture_output=True)
        assert verified.returncode == 0, name + ': ' + verified.stdout + verified.stderr
        command[1] = 'run'
        executed = subprocess.run(command, text=True, capture_output=True)
        assert executed.returncode == 0 and executed.stdout == expected, name + ': ' + executed.stdout + executed.stderr
        results[name] = 'compiled by rvnc, imported, verified and executed'

    case, missing = compile_case('missing-host-api', 'func Main() { let client = System.Net.Http.HttpClient() }\n')
    assert missing.returncode != 0 and not (case / 'compiled/Demo.dll').exists(), missing.stdout + missing.stderr
    results['missing-host-api'] = 'rejected without framework fallback or emitted assembly'
    wrong_core = project_text.replace('<RavenTargetCoreAssemblyName>NeoCLR.CoreProbe</RavenTargetCoreAssemblyName>',
                                      '<RavenTargetCoreAssemblyName>Other.Core</RavenTargetCoreAssemblyName>')
    assert wrong_core != project_text, 'Project must explicitly select the emission core.'
    case, wrong = compile_case('conflicting-core', 'func Main() { }\n', wrong_core)
    assert wrong.returncode != 0 and 'RAVT003' in wrong.stdout + wrong.stderr and not (case / 'compiled/Demo.dll').exists(), wrong.stdout + wrong.stderr
    results['conflicting-core'] = 'RAVT003; no emitted assembly'
print(json.dumps(results, indent=2))
