#!/usr/bin/env python3
"""Relink a JSON probe object with diagnostic caller/size and pacing counters."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
CALLERS = ROOT / "benchmarks/http-server/serve/caller-profile-20261018/callers.c"
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--build', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
build, out = a.build.resolve(), a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
manifest = json.loads((build/'build.json').read_text())
assert manifest['passed']
functions = json.loads(next(c['stdout'] for c in manifest['commands'] if c['phase']=='native-compilation'))['functions']
source = (ROOT/'runtime/native/gc/native-gc.c').read_text()
def replace(old, new):
    global source
    assert source.count(old) == 1, old
    source = source.replace(old, new)
replace('#include "native-gc.h"', '#include "native-gc.h"\nvoid record_caller(uint32_t, uint64_t);\nvoid record_boundary(unsigned);\nvoid record_idle(void);')
replace('    statistics.allocations++;', '    statistics.allocations++;\n    record_caller(kind, bytes);')
replace('int32_t neoclr_gc_idle_v1(neoclr_aot_context *context, const neoclr_probe_frame *boundary) {',
        'int32_t neoclr_gc_idle_v1(neoclr_aot_context *context, const neoclr_probe_frame *boundary) {\n    record_idle();')
replace('    return neoclr_gc_idle_v1(context, boundary);',
        '    record_boundary(pacing.context != context || statistics.collections != pacing.collections ? 0 :\n        statistics.allocations - pacing.allocations >= 16 ? 1 : 2);\n    return neoclr_gc_idle_v1(context, boundary);')
(out/'native-gc.c').write_text(source)
command = next(c['command'][:] for c in reversed(manifest['commands']) if c['phase']=='host-compile-link')
assert sum(x.endswith('/aot-console/native-gc.c') for x in command) == 1
command = [str(out/'native-gc.c') if x.endswith('/aot-console/native-gc.c') else x for x in command]
command[command.index('-o')+1] = str(out/'app')
command += ['-I', str(ROOT/'runtime/native/gc'), str(CALLERS)]
r = subprocess.run(command, capture_output=True, text=True)
inputs = [build/'app.o', build/'build.json', out/'native-gc.c', CALLERS, Path(__file__)]
inputs += [Path(x) for x in command if Path(x).is_file()]
report = dict(command=command, exitCode=r.returncode, stderr=r.stderr,
    inputs={str(x):hashlib.sha256(x.read_bytes()).hexdigest() for x in inputs})
(out/'build.json').write_text(json.dumps(report, indent=2)+'\n')
r.check_returncode()
result = subprocess.run([str(out/'app')], env=dict(os.environ, NEOCLR_ALLOCATION_CALLERS=str(out/'callers.json')), capture_output=True, text=True, timeout=180)
(out/'run.json').write_text(json.dumps(dict(exitCode=result.returncode, stdout=result.stdout, stderr=result.stderr), indent=2)+'\n')
result.check_returncode()
assert result.stdout == 'PASS JSON byte output probe\n'
names = {f['compiledIndex']:f['name'] for f in functions}
def readable(name):
    name = re.sub(r'NeoMetadata_[0-9A-F]+\.', '', name)
    return re.sub(r'(?<=[_])[0-9A-F]{2,}', lambda m: bytes.fromhex(m[0]).decode('utf-8',errors='replace'), name)
data = json.loads((out/'callers.json').read_text())
assert sum(r['count'] for r in data['rows'])+data['dropped'] == data['allocations']
assert sum(r['count']*r['size'] for r in data['rows'])+data['droppedBytes'] == data['allocatedBytes']
assert data['dropped'] == 0, 'Diagnostic table overflow; enlarge and rerun'
for row in data['rows']:
    for frame in row['frames']:
        frame['name'] = names.get(frame['function'], '<unmapped>')
        frame['displayName'] = readable(frame['name'])
data['rows'].sort(key=lambda r:r['count'], reverse=True)
data['method'] = 'Four published frames, including instruction IDs; truncated chains explicitly marked. Whole-process counts including startup/shutdown. Allocation payload sizes, not array element counts. Pacing reasons: context/epoch, >=16 allocations, >=64 boundaries. Instrumented timing is not comparative.'
(out/'summary.json').write_text(json.dumps(data,indent=2)+'\n')
print({k:v for k,v in data.items() if k != 'rows'})
for r in data['rows'][:15]:
    print(r['count'],r['size'],r['kind'],' -> '.join(f['displayName'] for f in r['frames']))
