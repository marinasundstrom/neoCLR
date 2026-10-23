#!/usr/bin/env python3
"""Validate native map admission, changing lane values and legacy fallback."""
import argparse
import hashlib
import json
from pathlib import Path
import signal
import subprocess
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--output',type=Path,required=True)
a=p.parse_args();a.output.mkdir(parents=True,exist_ok=False)
root=Path(__file__).resolve().parents[4];base=root/'docs/experiments/aot-console'
r=dict(passed=False,commands=[],inputs={})
try:
    for sanitize in [False,True]:
        binary=a.output.resolve()/str(sanitize)
        sources=[base/'native-root-map-test.c',base/'native-gc.c',base/'root-probe.c']
        cmd=['xcrun','--sdk','macosx','clang','-std=c11','-O2','-Wall','-Wextra','-Werror','-DNEOCLR_NATIVE_GC',*map(str,sources),'-o',str(binary)]
        if sanitize:cmd+=['-fsanitize=undefined,bounds','-fno-sanitize-recover=all']
        result=subprocess.run(cmd,capture_output=True,text=True,timeout=60)
        r['commands'].append(dict(command=cmd,exitCode=result.returncode,stderr=result.stderr));result.check_returncode()
        for case in ['valid','magic','count','length','null','head']:
            cmd=[str(binary),case];result=subprocess.run(cmd,capture_output=True,text=True,timeout=30)
            r['commands'].append(dict(command=cmd,exitCode=result.returncode,stderr=result.stderr))
            assert result.returncode==(0 if case=='valid' else -signal.SIGABRT)
    for path in [Path(__file__).resolve(),*sources,*(root/'runtime/native/gc').glob('*.[ch]')]:
        r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
    r['passed']=True
finally:
    (a.output/'validation.json').write_text(json.dumps(r,indent=2)+'\n')
