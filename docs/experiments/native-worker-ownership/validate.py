#!/usr/bin/env python3
"""Qualify the current native thread ownership boundary, without concurrent shared access."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import subprocess

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--address-sanitizer', action='store_true', help='Also attempt ASan/UBSan; retain unsupported/time-out outcomes as failures')
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
out = a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
here = Path(__file__).resolve().parent
root = here.parents[2]
sources = [root/'runtime/native/gc/native-gc.c', root/'runtime/native/gc/root-probe.c', here/'probe.c']
report = dict(passed=False, platform=platform.platform(), commands=[],
              meaning='Current POSIX owner-local root boundary only; no shared-heap parallelism or throughput claim.',
              inputs={str(f.relative_to(root)):hashlib.sha256(f.read_bytes()).hexdigest() for f in
                      [*sources, *list((root/'runtime/native/gc').glob('*.h')), here/'validate.py']})
cc = ['clang']
if platform.system() == 'Darwin':
    cc = [subprocess.check_output(['xcrun', '--find', 'clang'], text=True).strip(),
          '-isysroot', subprocess.check_output(['xcrun', '--show-sdk-path'], text=True).strip()]
try:
    variants = [('optimized', []), ('ubsan', ['-fsanitize=undefined', '-fno-sanitize-recover=all'])]
    if a.address_sanitizer:
        variants.append(('asan', ['-fsanitize=address,undefined', '-fno-omit-frame-pointer']))
    for name, extra in variants:
        command = [*cc,'-std=c11','-O2','-Wall','-Wextra','-Werror','-pthread','-DNEOCLR_NATIVE_GC',
                   '-I',str(root/'runtime/native/gc'), *map(str,sources),*extra,'-o',str(out/name)]
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
print('PASS native ownership probe in selected builds')
