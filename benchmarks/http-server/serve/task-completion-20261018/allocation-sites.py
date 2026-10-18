#!/usr/bin/env python3
"""Diagnostic allocation counts by published guest frame, never capacity timing."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--build', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
root = Path(__file__).resolve().parents[4]
build, out = a.build.resolve(), a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
manifest = json.loads((build/'build.json').read_text())
assert manifest['passed']
functions = json.loads(next(c['stdout'] for c in manifest['commands'] if c['phase']=='native-compilation'))['functions']
source = (root/'runtime/native/gc/native-gc.c').read_text()
hook = r'''
#include <stdio.h>
#include <stdlib.h>
#include <inttypes.h>
static uint64_t site_count[NEOCLR_NATIVE_FUNCTION_LIMIT + 1];
static uint64_t site_bytes[NEOCLR_NATIVE_FUNCTION_LIMIT + 1];
static void record_site(uint64_t bytes) {
    const neoclr_probe_frame *frame = neoclr_root_probe_head_v1();
    unsigned id = frame ? frame->function : NEOCLR_NATIVE_FUNCTION_LIMIT;
    if (id > NEOCLR_NATIVE_FUNCTION_LIMIT) abort();
    site_count[id]++;
    site_bytes[id] += bytes;
}
static void report_sites(void) {
    const char *path = getenv("NEOCLR_ALLOCATION_SITES");
    if (!path) abort();
    FILE *file = fopen(path, "w");
    if (!file) abort();
    fputs("[", file);
    int comma = 0;
    for (unsigned i = 0; i <= NEOCLR_NATIVE_FUNCTION_LIMIT; i++) {
        if (!site_count[i]) continue;
        fprintf(file, "%s{\"function\":%u,\"allocations\":%" PRIu64 ",\"bytes\":%" PRIu64 "}",
                comma ? "," : "", i, site_count[i], site_bytes[i]);
        comma = 1;
    }
    fputs("]\n", file);
    if (fclose(file)) abort();
}
__attribute__((constructor)) static void initialize_sites(void) {
    if (atexit(report_sites)) abort();
}
'''
assert source.count('    statistics.allocations++;') == 1
source = source.replace('#include "native-gc.h"', '#include "native-gc.h"\n'+hook)
source = source.replace('    statistics.allocations++;', '    statistics.allocations++;\n    record_site(bytes);')
(out/'native-gc.c').write_text(source)
command = next(c['command'][:] for c in reversed(manifest['commands']) if c['phase']=='host-compile-link')
command = [str(out/'native-gc.c') if x.endswith('/aot-console/native-gc.c') else x for x in command]
command[command.index('-o')+1] = str(out/'app')
command += ['-I', str(root/'runtime/native/gc')]
result = subprocess.run(command, capture_output=True, text=True)
(out/'build.json').write_text(json.dumps(dict(command=command, exitCode=result.returncode, stderr=result.stderr,
    inputs={str(x): hashlib.sha256(x.read_bytes()).hexdigest() for x in [build/'app.o', build/'build.json', out/'native-gc.c', Path(__file__)]}), indent=2)+'\n')
result.check_returncode()
env = dict(os.environ, NEOCLR_ALLOCATION_SITES=str(out/'sites.json'))
subprocess.run(['python3', str(root/'benchmarks/http-server/serve/diagnose.py'), '--executable', str(out/'app'), '--rate', '20', '--duration', '10', '--output', str(out/'requests')], env=env, check=True)
names = {f['compiledIndex']: f['name'] for f in functions}
def readable(name):
    name = re.sub(r'NeoMetadata_[0-9A-F]+\.', '', name)
    return re.sub(r'(?<=[_])[0-9A-F]{2,}', lambda m: bytes.fromhex(m[0]).decode('utf-8', errors='replace'), name)
rows = json.loads((out/'sites.json').read_text())
for row in rows:
    row['name'] = names.get(row['function'], '<no published guest frame>')
    row['displayName'] = readable(row['name'])
rows.sort(key=lambda r: r['allocations'], reverse=True)
(out/'summary.json').write_text(json.dumps(dict(method='Allocation charged to current published frame, including startup/shutdown. This is a caller attribution, not an allocated-type census or CPU profile.', rows=rows), indent=2)+'\n')
for row in rows[:15]: print(row['allocations'], row['bytes'], row['displayName'])
