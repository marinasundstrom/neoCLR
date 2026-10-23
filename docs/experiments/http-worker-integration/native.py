#!/usr/bin/env python3
"""Qualify the private listener group with optimized and UBSan native hosts."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--validation', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
here = Path(__file__).resolve().parent
root = here.parents[2]
out = a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
command = json.loads(a.validation.read_text())['commands'][0]['command']
command = [str(here/'native-test.c') if x.endswith('/test.c') else x for x in command]
# socket-os.h remains the existing adapter's platform abstraction.
command += [str(root/'runtime/native/workers/http-workers.c'), '-I', str(root/'docs/experiments/aot-console')]
report = dict(passed=False, commands=[], inputs={})
for path in [*list((root/'runtime/native/workers').glob('*.[ch]')), here/'native-test.c', Path(__file__).resolve(), root/'docs/experiments/aot-console/socket-os.h']:
    report['inputs'][str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()
try:
    for variant, flags in [('optimized', []), ('ubsan', ['-fsanitize=undefined', '-fno-sanitize-recover=all'])]:
        cmd = command.copy()
        cmd[cmd.index('-o')+1] = str(out/variant)
        for invocation in [cmd + flags, [str(out/variant)]]:
            r = subprocess.run(invocation, capture_output=True, text=True, timeout=60)
            report['commands'].append(dict(command=invocation, exitCode=r.returncode, stdout=r.stdout, stderr=r.stderr))
            r.check_returncode()
    report['passed'] = True
finally:
    (out/'validation.json').write_text(json.dumps(report, indent=2)+'\n')
print('PASS isolated HTTP listener group in optimized and UBSan builds')
