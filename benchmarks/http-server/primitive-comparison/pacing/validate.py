#!/usr/bin/env python3
"""Check each retained experimental collector with pacing and pressure contracts."""
import argparse
import hashlib
import json
import re
from pathlib import Path
import subprocess
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--experiment',type=Path,required=True)
a = p.parse_args()
root = Path(__file__).resolve().parents[4]
base = root/'docs/experiments/aot-console'
out = a.experiment.resolve()
report = dict(passed=False, commands=[], inputs={})
def record(path):
    report['inputs'][str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()
def run(cmd):
    r = subprocess.run(list(map(str,cmd)),capture_output=True,text=True,timeout=60)
    report['commands'].append(dict(command=list(map(str,cmd)),exitCode=r.returncode,stdout=r.stdout,stderr=r.stderr))
    r.check_returncode()
    return r.stdout.strip()
try:
    cc = run(['xcrun','--sdk','macosx','--find','clang'])
    sdk = run(['xcrun','--sdk','macosx','--show-sdk-path'])
    record(Path(__file__).resolve())
    original = base/'native-gc-pacing-test.c'
    record(original)
    for allocations,boundaries in [(16,64),(64,256),(256,1024)]:
        gc = out/f'gc-{allocations}.c'
        record(gc)
        fixture = out/f'pacing-{allocations}.c'
        text = original.read_text()
        text, count = re.subn(r'ALLOCATION_BUDGET = \d+, PUBLICATION_BUDGET = \d+', f'ALLOCATION_BUDGET = {allocations}, PUBLICATION_BUDGET = {boundaries}', text)
        assert count == 1
        fixture.write_text(text)
        for source in [fixture,base/'native-gc-admission-test.c']:
            for sanitizer in [False,True]:
                binary = out/f'{source.stem}-{allocations}-{sanitizer}'
                sources = [source,gc,base/'root-probe.c',base/'text-arena.c']
                for path in sources: record(path)
                cmd = [cc,'-isysroot',sdk,'-std=c11','-O2','-Wall','-Wextra','-Werror','-DNEOCLR_NATIVE_GC','-DNEOCLR_GC_PACED','-I',base,'-I',root/'runtime/native/gc',*sources,'-o',binary]
                if sanitizer: cmd += ['-fsanitize=undefined,bounds','-fno-sanitize-recover=all']
                run(cmd)
                run([binary])
    report['passed'] = True
finally:
    (out/'validation.json').write_text(json.dumps(report,indent=2)+'\n')
