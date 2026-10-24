#!/usr/bin/env python3
"""Use a qualified historical worker/package with only the interpreter replaced."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

root=Path(__file__).resolve().parents[3]
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--directory',type=Path,required=True)
a=p.parse_args();out=a.directory.resolve()
source=root/'benchmarks/execution/candidate-refreshed-20261017/manifest.json'
prior=json.loads(source.read_text())
binary=str(root/'target/release/neoclr')
# Native source build recipes are not executed by this interpreter-only run.
# Keep all retained package/application inputs checked; do not bless changed DLLs.
inputs={k:v for k,v in prior['inputs'].items() if Path(k).is_relative_to(root/'target')}
for path,digest in inputs.items():
    assert hashlib.sha256(Path(path).read_bytes()).hexdigest()==digest,path
inputs[binary]=hashlib.sha256(Path(binary).read_bytes()).hexdigest()
manifest=dict(schemaVersion=1,cases=prior['cases'],host=prior['host'],cpu=prior['cpu'],
              commands=dict(interpreter=[binary,*prior['commands']['interpreter'][1:]]),inputs=inputs,
              provenance=dict(source=str(source),sourceSha256=hashlib.sha256(source.read_bytes()).hexdigest(),
              override='Only interpreter executable replaced. All retained package/application inputs unchanged.',
              excludedBuildOnlyInputs={k:v for k,v in prior['inputs'].items() if k not in inputs}))
path=out/'interpreter-qualified-manifest.json';assert not path.exists()
path.write_text(json.dumps(manifest,indent=2)+'\n')
cmd=['python3',str(root/'benchmarks/framework/runner.py'),'--manifest',str(path),'--output',str(out/'interpreter-batches.json'),
     '--mode','interpreter','--rounds','3','--warmups','2','--operations','16','--timeout','120']
x=subprocess.run(cmd,capture_output=True,text=True,timeout=240)
(out/'interpreter-run.json').write_text(json.dumps(dict(command=cmd,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr),indent=2)+'\n')
print(x.stdout,flush=True);x.check_returncode()
