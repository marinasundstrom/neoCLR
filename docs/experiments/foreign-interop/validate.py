#!/usr/bin/env python3
"""Compile actual Raven interop attributes and consumers, then execute native imports."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess

ROOT = Path(__file__).resolve().parents[3]
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--raven', type=Path, required=True)
p.add_argument('--core', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
p.add_argument('--report', type=Path, required=True)
a = p.parse_args()
output = a.output.resolve()
output.mkdir(parents=True, exist_ok=True)
report = {'host': platform.platform(), 'cases': [], 'commands': []}
def run(command):
    result = subprocess.run(list(map(str, command)), cwd=ROOT, capture_output=True, text=True, timeout=300)
    report['commands'].append({'command': list(map(str, command)), 'exitCode': result.returncode, 'stdout': result.stdout, 'stderr': result.stderr})
    return result
library = output / ('foreign_fixture.dll' if os.name == 'nt' else 'libforeign_fixture.dylib' if platform.system() == 'Darwin' else 'libforeign_fixture.so')
fixture = run(['rustc', '--edition=2024', '--crate-type=cdylib', '--crate-name=foreign_fixture', ROOT / 'examples/native/library.rs', '-o', library])
assert fixture.returncode == 0, fixture.stderr
probe = a.raven.resolve() / 'tools/NeoClrMetadataProbe/bin/Debug/net10.0/NeoClrMetadataProbe.dll'
attribute = ROOT / 'runtime/raven/src/System/Runtime/InteropServices/NativeImportAttribute.rvn'
emitted = run(['dotnet', probe, '--native-foreign-source', a.core.resolve(), library, output, attribute])
assert emitted.returncode == 0, emitted.stdout + emitted.stderr
runtime = ROOT / 'target/debug' / ('neoclr.exe' if os.name == 'nt' else 'neoclr')
for name, expected in [('ForeignAdd', 42), ('ForeignStrings', 42), ('ForeignWide', 1), ('ForeignDefaultName', 42), ('ForeignSystem', 42), ('ForeignMixedEncoding', 42), ('ForeignBool8', 42), ('ForeignBool32', 42), ('ForeignStatic', 42), ('ForeignMissing', 1)]:
    artifact = output / (name + '.dll')
    checked = run([runtime, 'check', artifact])
    assert checked.returncode == 0, checked.stderr
    result = run([runtime, 'run', artifact, '--execution', 'interpreter'])
    missing = name == 'ForeignMissing'
    passed = result.returncode == expected and (('RuntimeError' in result.stderr and 'instruction' in result.stderr) if missing else not result.stderr)
    report['cases'].append({'name': name, 'expectedExit': expected, 'exitCode': result.returncode, 'passed': passed})
    assert passed, result.stdout + result.stderr
report['compilerBaseRevision'] = subprocess.check_output(['git', '-C', str(a.raven), 'rev-parse', 'HEAD'], text=True).strip()
report['runtimeBaseRevision'] = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
report['note'] = 'Working-tree compiler/runtime changes; base revisions do not identify these uncommitted implementations.'
report['inputs'] = {str(path): hashlib.sha256(path.read_bytes()).hexdigest() for path in [attribute, probe, runtime, a.core.resolve(), ROOT / 'src/interop.rs', ROOT / 'src/metadata.rs']}
a.report.parent.mkdir(parents=True, exist_ok=True)
a.report.write_text(json.dumps(report, indent=2) + '\n')
print(f"PASS {len(report['cases'])} native execution cases; compiler probe also checks rejected declarations")
