#!/usr/bin/env python3
"""Validate bounded native workers and explicit gated heap ownership."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import subprocess

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--address-sanitizer', action='store_true', help='Also attempt ASan/UBSan; retain unsupported/time-out outcomes as failures')
p.add_argument('--compiled-workers', action='store_true', help='Compile the optional shared compiler coordinator')
p.add_argument('--thread-sanitizer', action='store_true')
p.add_argument('--trace-workers', action='store_true', help='Compile optional worker trace hooks; ordinary constructor still disables recording')
p.add_argument('--local-queues', action='store_true', help='Select the bounded local/global/stealing experiment')
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
out = a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
here = Path(__file__).resolve().parent
root = here.parents[2]
sources = [root/'runtime/native/gc/native-gc.c', root/'runtime/native/gc/root-probe.c', here/'test.c', root/'runtime/native/workers/native-workers.c']
report = dict(passed=False, platform=platform.platform(), commands=[],
              meaning='POSIX bounded pool with gated managed access and explicit capture/result roots; no generated-code or HTTP integration.',
              inputs={str(f.relative_to(root)):hashlib.sha256(f.read_bytes()).hexdigest() for f in
                      [*sources, *list((root/'runtime/native/gc').glob('*.h')), *list((root/'runtime/native/workers').glob('*.h')), root/'runtime/native/workers/compiled-workers.inc', here/'benchmark.c', here/'validate.py']})
cc = ['clang']
if platform.system() == 'Darwin':
    cc = [subprocess.check_output(['xcrun', '--find', 'clang'], text=True).strip(),
          '-isysroot', subprocess.check_output(['xcrun', '--show-sdk-path'], text=True).strip()]
try:
    variants = [('optimized', []), ('ubsan', ['-fsanitize=undefined', '-fno-sanitize-recover=all'])]
    if a.address_sanitizer:
        variants.append(('asan', ['-fsanitize=address,undefined', '-fno-omit-frame-pointer']))
    if a.thread_sanitizer:
        variants.append(('tsan', ['-fsanitize=thread', '-fno-omit-frame-pointer']))
    for name, extra in variants:
        command = [*cc,'-std=c11','-O2','-Wall','-Wextra','-Werror','-pthread','-DNEOCLR_NATIVE_GC',
                   '-I',str(root/'runtime/native/gc'), '-I', str(root/'runtime/native/workers'), *map(str,sources),*extra,'-o',str(out/name)]
        if a.compiled_workers: command.append('-DNEOCLR_WORKER_GC')
        if a.trace_workers: command.append('-DNEOCLR_WORKER_TRACE')
        if a.local_queues: command.append('-DNEOCLR_WORKER_QUEUES')
        for cmd in [command, [str(out/name)]]:
            try:
                result = subprocess.run(cmd,capture_output=True,text=True,timeout=60)
            except subprocess.TimeoutExpired:
                report['commands'].append(dict(command=cmd, outcome='timeout', timeoutSeconds=60))
                raise
            report['commands'].append(dict(command=cmd,exitCode=result.returncode,stdout=result.stdout,stderr=result.stderr))
            result.check_returncode()
    report['passed'] = True
finally:
    (out/'validation.json').write_text(json.dumps(report,indent=2)+'\n')
print('PASS native worker pool in selected builds')
