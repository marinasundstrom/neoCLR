#!/usr/bin/env python3
import argparse
import json
import hashlib
from pathlib import Path
import subprocess
p = argparse.ArgumentParser()
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
a.output.mkdir(parents=True, exist_ok=False)
root = Path(__file__).resolve().parents[4]
checks=[]
for sanitized in [False, True]:
    for mode, defines in [('none', []), ('gc', ['NEOCLR_NATIVE_GC']), ('async', ['NEOCLR_RUNTIME_ASYNC']), ('full', ['NEOCLR_NATIVE_GC','NEOCLR_GC_PROFILE','NEOCLR_RUNTIME_ASYNC'])]:
        binary=(a.output/f'{mode}-{sanitized}').resolve()
        command=['clang','-std=c11','-O2','-Wall','-Wextra','-Werror','-pthread', *['-D'+s for s in defines],
                 *sum([['-I',str(root/'runtime/native'/s)] for s in ['gc','async','diagnostics']],[]),
                 str(root/'runtime/native/diagnostics/snapshot.c'), str(Path(__file__).with_name('check.c')), '-o',str(binary)]
        if sanitized: command+=['-fsanitize=undefined,bounds','-fno-sanitize-recover=all']
        subprocess.run(command,check=True)
        subprocess.run([binary],check=True)
        checks.append(dict(command=command,passed=True))
source=Path('runtime/native/workers/http-workers.c')
old=a.output/'before.c'
old.write_bytes(subprocess.run(['git','show','HEAD:'+str(source)],check=True,capture_output=True).stdout)
objects=[]
for tag,path in [('before',old),('after',root/source)]:
    obj=a.output/f'{tag}.o'
    subprocess.run(['clang','-std=c11','-O2','-I',str(root/source.parent),'-I',str(root/'docs/experiments/aot-console'),'-c',str(path),'-o',str(obj)],check=True)
    objects.append(obj.read_bytes())
assert objects[0]==objects[1], 'Disabled instrumentation changed worker object'
(a.output/'validation.json').write_text(json.dumps(dict(passed=True,checks=checks,disabledWorkerObjectIdentical=True,baselineRevision=subprocess.run(['git','rev-parse','HEAD'],check=True,capture_output=True,text=True).stdout.strip(),inputs={str(path):hashlib.sha256(path.read_bytes()).hexdigest() for path in [root/'runtime/native/diagnostics/snapshot.c',root/'runtime/native/diagnostics/snapshot.h',root/source,Path(__file__),Path(__file__).with_name('check.c')]}),indent=2)+'\n')
print('Eight snapshot configurations passed; disabled worker object identical')
