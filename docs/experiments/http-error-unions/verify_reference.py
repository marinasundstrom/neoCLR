#!/usr/bin/env python3
"""Project an existing error reference from normal union syntax; no public migration."""
import argparse
import os
from pathlib import Path
import subprocess
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--bundle', required=True, type=Path)
parser.add_argument('--bridge', required=True, type=Path)
args = parser.parse_args()
bundle = args.bundle.resolve()
bridge = args.bridge.resolve()
env = dict(os.environ, NeoCLRRoot=str(bundle), RavenSdkRoot=str(bundle / 'raven-sdk'))
owner = 'System.Networking.Sockets.SocketError'
cases = ['Closed', 'Busy', 'InvalidRange', 'LimitExceeded', 'Cancelled', 'InvalidAddress',
         'ConnectionRefused', 'ConnectionReset', 'AccessDenied', 'TimedOut', 'IoFailure',
         'InvalidOperation', 'AddressInUse']


def run(command, success=True):
    result = subprocess.run([str(x) for x in command], env=env, capture_output=True,
                            text=True, timeout=180)
    if success and result.returncode:
        raise AssertionError(result.stdout + result.stderr)
    return result


with tempfile.TemporaryDirectory(prefix='neoclr-union-reference-') as directory:
    root = Path(directory)
    core = bundle / 'demo/NeoCLR.CoreProbe.dll'
    (root / 'Errors.rvn').write_text('namespace System.Networking.Sockets\n'
        'public union SocketError {\n' + ''.join('    case ' + case + '\n' for case in cases)
        + '    override func ToString() -> string {\n'
        + ''.join(f'        if let SocketError.{case} = self {{\n            return "{case}"\n        }}\n'
                  for case in cases)
        + '        return "Empty"\n    }\n}\n')

    def compile_source(name, files, reference):
        project = root / (name + '.rvnproj')
        project.write_text('<Project>\n<Import Project="$(NeoCLRRoot)/build/NeoCLR.Raven.props" />\n'
            '<PropertyGroup><OutputType>Library</OutputType></PropertyGroup>\n<ItemGroup>\n'
            f'<Reference Update="NeoCLR.CoreProbe"><HintPath>{reference}</HintPath></Reference>\n'
            + ''.join(f'<Compile Include="{file}" />\n' for file in files) + '</ItemGroup>\n</Project>')
        output = root / name
        run(['dotnet', bundle / 'raven-sdk/tools/rvnc/rvnc.dll', project,
             '--no-project-restore', '-o', output])
        return output / (name + '.dll')

    legacy_core = root / 'legacy/NeoCLR.CoreProbe.dll'
    legacy_core.parent.mkdir()
    run(['dotnet', bridge, '--reference-library-core', legacy_core])
    repository = Path(__file__).resolve().parents[3]
    legacy_source = compile_source('Legacy', [repository / 'runtime/raven/src/System/Int32ParseError.rvn'], legacy_core)
    run(['dotnet', bridge, '--library-implementation', legacy_source, legacy_core, 'System.Int32ParseError', root / 'legacy-native'])
    assert '.field private Stored Value' in (root / 'legacy-native/Implementation.neoil').read_text()
    print('Existing erased Int32ParseError implementation still imports.')
    source = compile_source('Source', ['Errors.rvn'], core)
    projected = root / 'projected/NeoCLR.CoreProbe.dll'
    projected.parent.mkdir()
    run(['dotnet', bridge, '--project-union-reference', source, core, owner, projected])
    report = run(['dotnet', bridge, '--union-metadata-report', projected]).stdout
    assert all('"Name":"' + case + '"' in report for case in cases), report
    (root / 'Consumer.rvn').write_text('''import System.*
import System.Tasks.*
import System.Networking.Sockets.*
func Connect() -> Task<Result<Socket, SocketError>> {
    Socket.Connect("127.0.0.1", 80)
}
func Inspect(value: SocketError) -> bool {
    if let SocketError.Closed = value {
        return true
    }
    return false
}
func Create() -> SocketError {
    SocketError.AddressInUse
}
''')
    compile_source('Consumer', ['Consumer.rvn'], projected)
    # Repeated projection must retain one shared protocol and preserve core signatures.
    shared_source = compile_source('SharedSource', ['Errors.rvn'], projected)
    repeated = root / 'repeated/NeoCLR.CoreProbe.dll'
    repeated.parent.mkdir()
    run(['dotnet', bridge, '--project-union-reference', shared_source, projected, owner, repeated])
    assert run(['dotnet', bridge, '--union-metadata-report', repeated]).stdout == report
    compile_source('RepeatedConsumer', ['Consumer.rvn'], repeated)
    run(['dotnet', bridge, '--library-implementation', shared_source, repeated, owner, root / 'native'])
    body = (root / 'native/Implementation.neoil').read_text()
    assert '.type System.Networking.Sockets.SocketError' in body
    assert '.field private Stored Value' not in body
    assert '.interface System.Runtime.CompilerServices.IUnion' not in body
    assert 'Raven.Runtime.CompilerServices' not in body
    assert 'out(true)' in body
    altered = root / 'mismatched.dll'
    run(['dotnet', bridge, '--standard-union-library-core', source, core, owner, altered, 'wrong-case'])
    rejected_output = root / 'rejected.dll'
    rejected = run(['dotnet', bridge, '--project-union-reference', source, altered, owner, rejected_output], success=False)
    assert rejected.returncode != 0 and 'case identities do not match' in rejected.stderr, rejected.stdout + rejected.stderr
    assert not rejected_output.exists()
    print('SocketError reference replacement, shared reprojection, consumer signatures and native import passed.')
    print('Reference replacement and shared support remain consistent with the migrated SocketError.')
