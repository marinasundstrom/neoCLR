#!/usr/bin/env python3
"""Check whole-process diagnostic AOT reclamation for both streaming receivers."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import validate
ROOT=Path(__file__).resolve().parents[3]
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--client',type=Path,required=True)
p.add_argument('--server',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
a=p.parse_args();out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
report=dict(passed=False,cases={},scope='Whole-process diagnostic GC including errors and cancellation, not throughput or allocations per request.')
try:
    for kind in ['client','server']:
        build_path=getattr(a,kind)/'build.json';build=json.loads(build_path.read_text())
        command=next(c['command'][:] for c in build['commands'] if c.get('phase')=='host-compile-link')
        binary=out/kind;gc=out/(kind+'.json')
        command[command.index('-o')+1]=str(binary)
        command+=['-DNEOCLR_GC_PROFILE',str(ROOT/'benchmarks/http-server/profile-gc.c')]
        inputs={str(Path(x).resolve()):hashlib.sha256(Path(x).read_bytes()).hexdigest() for x in command if Path(x).is_file()}
        for x in [Path(__file__),Path(validate.__file__),build_path]:inputs[str(x.resolve())]=hashlib.sha256(x.read_bytes()).hexdigest()
        subprocess.run(command,check=True,capture_output=True,text=True)
        result=validate.run_case(['/usr/bin/env','NEOCLR_HTTP_PROFILE_OUTPUT='+str(gc),str(binary)],kind)
        counters=json.loads(gc.read_text().splitlines()[0])
        assert counters['allocations']>0 and counters['failures']==0
        assert counters['allocations']==counters['reclaimed'],counters
        for path,digest in inputs.items():assert hashlib.sha256(Path(path).read_bytes()).hexdigest()==digest,path
        report['cases'][kind]=dict(validation=result,gc=counters,command=command,inputs=inputs)
        print(kind,counters,flush=True)
    report['passed']=True
finally:(out/'profile.json').write_text(json.dumps(report,indent=2)+'\n')
