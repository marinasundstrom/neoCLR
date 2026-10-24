"""Recompile retained standalone assemblies with a new live backend; reuse sync objects."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--build',type=Path,required=True)
p.add_argument('--runner',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
a=p.parse_args()
root=Path(__file__).resolve().parents[3]
out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
report=json.loads((a.build/'results.json').read_text())
report['passed']=False;report['scope']=__doc__
report['reusedSyncObjects']={}
try:
    for case in report['cases']:
        directory=out/case['sample'];directory.mkdir()
        previous=a.build.resolve()/case['sample']/'sync.o'
        shutil.copy2(previous,directory/'sync.o')
        report['reusedSyncObjects'][str(directory/'sync.o')]=str(previous)
        live=next(n for n in case['native'] if n['mode']=='live')
        command=live['selectionCommand'].copy();command[4]=str(directory/'live.o')
        path=directory/'live-command.json';path.write_text(json.dumps(command,indent=2)+'\n')
        invocation=[str(a.runner.resolve()),'mutation_store::tests::full_http_selection_compiles_with_live_gc','--exact','--ignored','--nocapture']
        result=subprocess.run(invocation,cwd=root,env=dict(os.environ,NEOCLR_LIVE_SELECTION_COMMAND=str(path),NEOCLR_LIVE_SELECTION_MODE='live'),capture_output=True,text=True,timeout=300)
        live.update(command=invocation,selectionCommand=command,exitCode=result.returncode,stdout=result.stdout,stderr=result.stderr)
        for item in [*command, str(directory/'sync.o')]:
            candidate=Path(item)
            if candidate.is_file():report['inputs'][str(candidate)]=hashlib.sha256(candidate.read_bytes()).hexdigest()
        (out/'results.json').write_text(json.dumps(report,indent=2)+'\n')
    for path in [a.build/'results.json',a.runner,Path(__file__),root/'runtime/codegen/src/mutation_store.rs',root/'runtime/codegen/src/mutation_store_tests.rs']:
        report['inputs'][str(path.resolve())]=hashlib.sha256(path.read_bytes()).hexdigest()
    report['passed']=all(n['exitCode']==0 for c in report['cases'] for n in c['native'])
finally:
    (out/'results.json').write_text(json.dumps(report,indent=2)+'\n')
raise SystemExit(0 if report['passed'] else 1)
