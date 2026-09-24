#!/usr/bin/env python3
"""Exercise the migrated core SocketError inside a normal source union under GC."""
import argparse
import os
from pathlib import Path
import subprocess
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--bundle', required=True, type=Path)
parser.add_argument('--runner', required=True, type=Path)
args = parser.parse_args()
bundle = args.bundle.resolve()
env = dict(os.environ, NeoCLRRoot=str(bundle), RavenSdkRoot=str(bundle / 'raven-sdk'))


def run(command):
    result = subprocess.run([str(x) for x in command], env=env, capture_output=True,
                            text=True, timeout=180)
    assert result.returncode == 0, result.stdout + result.stderr
    return result


with tempfile.TemporaryDirectory(prefix='neoclr-socket-union-') as directory:
    root = Path(directory)
    (root / 'Errors.rvn').write_text((Path(__file__).parent / 'LegacyErrors.rvn').read_text())
    (root / 'Main.rvn').write_text('''import System.*
import System.Networking.Sockets.*
import HttpErrorProbe.*
func Check(value: bool) {
    if !value { System.Fault("Socket union assertion failed") }
}
func CheckName(value: SocketError, name: string) {
    Check(value.HasValue)
    Check(value.ToString() == name)
    let boxed: Object = value
    Check(boxed.ToString() == name)
}
func Main() {
    CASE_CHECKS
    let inactive = default(SocketError)
    Check(!inactive.HasValue)
    Check(inactive.ToString() == "Empty")
    Check(inactive.Value == null)
    let error: SocketError = SocketError.Closed
    if let SocketError.Closed = error { } else { System.Fault("Wrong case") }
    if let SocketError.Busy = error { System.Fault("Wrong match") }
    let preserved: Object = error
    var outer: HttpError = HttpError.Transport(error)
    let copy = outer
    outer = HttpError.TimedOut
    if let HttpError.Transport(cause) = copy {
        Check(cause.ToString() == "Closed")
    } else { System.Fault("Lost nested case") }
    Check(!default(HttpError).HasValue)
    var index = 0
    while index < 100 {
        let failure: HttpError = HttpError.Transport(SocketError.ConnectionReset)
        let boxed: Object = failure
        Check(boxed.ToString() == "ConnectionReset")
        Check(preserved.ToString() == "Closed")
        index += 1
    }
    Console.WriteLine("SocketError cases, nested unions, defaults and boxed copies passed")
}
'''.replace('CASE_CHECKS', '\n    '.join(
        f'CheckName(SocketError.{case}, "{case}")' for case in [
            'Closed', 'Busy', 'InvalidRange', 'LimitExceeded', 'Cancelled', 'InvalidAddress',
            'ConnectionRefused', 'ConnectionReset', 'AccessDenied', 'TimedOut', 'IoFailure',
            'InvalidOperation', 'AddressInUse'])))
    (root / 'Check.rvnproj').write_text('''<Project>
<Import Project="$(NeoCLRRoot)/build/NeoCLR.Raven.props" />
<ItemGroup><Compile Include="Errors.rvn" /><Compile Include="Main.rvn" /></ItemGroup>
</Project>''')
    run(['dotnet', bundle / 'raven-sdk/tools/rvnc/rvnc.dll', root / 'Check.rvnproj',
         '--no-project-restore', '-o', root / 'compiled'])
    run(['dotnet', bundle / 'tools/bridge/Probe.dll', '--import', root / 'compiled/Check.dll',
         bundle / 'demo/NeoCLR.CoreProbe.dll', root / 'imported'])
    result = run([args.runner.resolve(), root / 'imported/App.neoil', bundle / 'lib/System.neoil', '64', '10000000'])
    assert result.stdout.splitlines() == ['SocketError cases, nested unions, defaults and boxed copies passed']
    assert 'live=0' in result.stderr, result.stderr
    print(result.stdout + result.stderr)
