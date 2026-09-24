#!/usr/bin/env python3
"""Exercise migrated core error unions inside normal source unions under GC."""
import argparse
import os
import re
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


# Exercise each migrated production declaration through the consumer reference.
repository = Path(__file__).resolve().parents[3]
extra_checks = []
extra_functions = []
for path in sorted((repository / 'runtime/raven/src').rglob('*.rvn')):
    source = path.read_text()
    declaration = re.search(r'public union (\w+) \{', source)
    if declaration is None or declaration[1] in ('SocketError', 'DnsError', 'UriError'):
        continue
    name = declaration[1]
    owner = re.search(r'namespace (\S+)', source)[1] + '.' + name
    cases = re.findall(r'^    case (\w+)$', source, re.M)
    if not cases:
        continue
    extra_functions.append(f"""func Check{name}(value: {owner}, name: string) {{
    Check(value.HasValue)
    Check(value.ToString() == name)
    let boxed: Object = value
    Check(boxed.ToString() == name)
}}
""")
    extra_checks.extend(f'Check{name}({owner}.{case}, "{case}")' for case in cases)
    extra_checks.extend([f'let inactive{name} = default({owner})',
        f'Check(!inactive{name}.HasValue)', f'Check(inactive{name}.Value == null)',
        f'Check(inactive{name}.ToString() == "Empty")'])

with tempfile.TemporaryDirectory(prefix='neoclr-socket-union-') as directory:
    root = Path(directory)
    (root / 'Errors.rvn').write_text((Path(__file__).parent / 'LegacyErrors.rvn').read_text() + '''
public union LookupFailure {
    case Invalid(message: string)
    case Dns(cause: System.Networking.DnsError)
    case Address(cause: System.UriError)
}
''')
    (root / 'Main.rvn').write_text('''import System.*
import System.Networking.Sockets.*
import System.Networking.*
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
func CheckDns(value: DnsError, name: string) {
    Check(value.HasValue)
    Check(value.ToString() == name)
    let boxed: Object = value
    Check(boxed.ToString() == name)
}
func CheckUri(value: UriError, name: string) {
    Check(value.HasValue)
    Check(value.ToString() == name)
    let boxed: Object = value
    Check(boxed.ToString() == name)
}
EXTRA_FUNCTIONS
func Main() {
    EXTRA_CHECKS
    Check((int)System.Storage.EntryKind.File == 1)
    Check((int)System.Storage.EntryKind.Directory == 2)
    Check((int)default(System.Storage.EntryKind) == 0)
    Check(System.Storage.EntryKind.File != System.Storage.EntryKind.Directory)
    let boxedKind: Object = System.Storage.EntryKind.File
    Check((System.Storage.EntryKind)boxedKind == System.Storage.EntryKind.File)
    BATCH_CHECKS
    let inactiveDns = default(DnsError)
    Check(!inactiveDns.HasValue)
    Check(inactiveDns.Value == null)
    Check(inactiveDns.ToString() == "Empty")
    let inactiveUri = default(UriError)
    Check(!inactiveUri.HasValue)
    Check(inactiveUri.Value == null)
    Check(inactiveUri.ToString() == "Empty")
    var failure: LookupFailure = LookupFailure.Dns(DnsError.LookupFailed)
    let copied = failure
    failure = LookupFailure.Address(UriError.InvalidFormat)
    if let LookupFailure.Dns(cause) = copied {
        Check(cause is DnsError.LookupFailed)
        Check(!(cause is DnsError.InvalidName))
    } else {
        System.Fault("Lost DNS case")
    }
    if let LookupFailure.Address(cause) = failure {
        Check(cause is UriError.InvalidFormat)
        Check(!(cause is UriError.TooLong))
    } else {
        System.Fault("Lost URI case")
    }
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
    Console.WriteLine("Core error cases, nested unions, defaults and boxed copies passed")
}
'''.replace('EXTRA_FUNCTIONS', ''.join(extra_functions)).replace('EXTRA_CHECKS', '\n    '.join(extra_checks)).replace('BATCH_CHECKS', '\n    '.join(
        f'Check{family}({owner}.{case}, \"{case}\")'
        for family, owner, cases in [
            ('Dns', 'DnsError', ['InvalidName', 'LimitExceeded', 'NoAddress', 'TimedOut', 'Cancelled', 'LookupFailed']),
            ('Uri', 'UriError', ['InvalidFormat', 'UnsupportedAuthority', 'TooLong', 'BaseNotAbsolute'])]
        for case in cases)).replace('CASE_CHECKS', '\n    '.join(
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
    assert result.stdout.splitlines() == ['Core error cases, nested unions, defaults and boxed copies passed']
    assert 'live=0' in result.stderr, result.stderr
    print(result.stdout + result.stderr)
    # Raven overlays this all-value payload form. The bridge admits only empty-case
    # explicit layouts today; keep the distinct limitation visible and fail closed.
    errors = root / 'Errors.rvn'
    errors.write_text(errors.read_text().replace('    case Invalid(message: string)\n', ''))
    run(['dotnet', bundle / 'raven-sdk/tools/rvnc/rvnc.dll', root / 'Check.rvnproj',
         '--no-project-restore', '-o', root / 'overlaid'])
    rejected = subprocess.run(['dotnet', str(bundle / 'tools/bridge/Probe.dll'), '--import',
        str(root / 'overlaid/Check.dll'), str(bundle / 'demo/NeoCLR.CoreProbe.dll'),
        str(root / 'overlaid-import')], env=env, capture_output=True, text=True, timeout=180)
    assert rejected.returncode != 0 and 'Unsupported application type: HttpErrorProbe.LookupFailure' in rejected.stderr, rejected.stdout + rejected.stderr
    print('Payload-bearing explicit-layout union remains rejected')
