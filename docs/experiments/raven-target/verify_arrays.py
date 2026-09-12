"""Verify and execute the generated Raven array probe on neoCLR."""
import argparse
import json
from pathlib import Path
import subprocess

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('output', type=Path)
parser.add_argument('--runtime', required=True, type=Path)
args = parser.parse_args()
output, runtime = args.output.resolve(), args.runtime.resolve()
results = json.loads((output / 'array-results.json').read_text())
for name in ('CoreArrays', 'NullDefault'):
    check = subprocess.run([str(runtime), 'verify', str(output / (name + '.neoil'))], capture_output=True, text=True, timeout=30)
    if check.returncode:
        raise AssertionError(check.stdout + check.stderr)
run = subprocess.run([str(runtime), 'run', str(output / 'CoreArrays.neoil')], capture_output=True, text=True, timeout=30)
if run.returncode or run.stdout != '42\n2\n=> Void\n':
    raise AssertionError(run.stdout + run.stderr)
null = subprocess.run([str(runtime), 'run', str(output / 'NullDefault.neoil')], capture_output=True, text=True, timeout=30)
if null.returncode == 0 or 'null array reference' not in null.stderr:
    raise AssertionError(null.stdout + null.stderr)
results.update(Scope='Raven Int32 vector compile/import/verify/execute on neoCLR',
               RuntimeOutput=run.stdout, NullDefault='Verified; execution faults with null array reference')
(output / 'array-execution-results.json').write_text(json.dumps(results, indent=2) + '\n')
print(json.dumps(results, indent=2))
