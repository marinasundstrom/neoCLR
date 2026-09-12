"""Verify and execute imported Raven collection samples with the adapted System library."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('output', type=Path)
parser.add_argument('--runtime', required=True, type=Path)
parser.add_argument('--system', required=True, type=Path)
args = parser.parse_args()
output, runtime, system = args.output.resolve(), args.runtime.resolve(), args.system.resolve()
results = json.loads((output / 'interface-results.json').read_text())
if len(results.get('ImportRejections', {})) != 6 or len(results.get('NegativeDiagnostics', {})) != 2:
    raise AssertionError('Run the complete collection probe with rejection checks first')
for name in ('CoreInterfaces', 'CollectionAliases', 'CollectionForEach', 'NullCollection'):
    mapping = json.loads((output / (name + '.neoil.map.json')).read_text())
    assert mapping['RequiredLibraryProfile'] == 'raven-collections'
    check = subprocess.run([str(runtime), 'verify', str(output / (name + '.neoil')), '--system', str(system)],
                           capture_output=True, text=True, timeout=30)
    if check.returncode:
        raise AssertionError(check.stdout + check.stderr)
outputs = {}
for name, expected in [('CoreInterfaces', '1\n42\n=> Void\n'), ('CollectionAliases', '7\n42\n2\n=> Void\n'), ('CollectionForEach', '41\n42\n41\n41\n=> Void\n')]:
    run = subprocess.run([str(runtime), 'run', str(output / (name + '.neoil')), '--system', str(system)],
                         capture_output=True, text=True, timeout=30)
    if run.returncode or run.stdout != expected:
        raise AssertionError(run.stdout + run.stderr)
    outputs[name] = run.stdout
null = subprocess.run([str(runtime), 'run', str(output / 'NullCollection.neoil'), '--system', str(system)],
                      capture_output=True, text=True, timeout=30)
if null.returncode == 0 or 'null interface receiver' not in null.stderr:
    raise AssertionError(null.stdout + null.stderr)
results.update(Scope='Raven collection compile/import/verify/execute on neoCLR', RuntimeOutputs=outputs,
               SystemSha256=hashlib.sha256(system.read_bytes()).hexdigest(),
               NullDefault='Verified; execution faults with null interface receiver')
(output / 'collection-execution-results.json').write_text(json.dumps(results, indent=2) + '\n')
print(json.dumps(results, indent=2))
