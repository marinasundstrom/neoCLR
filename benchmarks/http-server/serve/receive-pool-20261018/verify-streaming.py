#!/usr/bin/env python3
"""Run the existing headers-first server cases with the same qualified mode commands."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--contracts',type=Path,required=True)
p.add_argument('--build',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
p.add_argument('--mode',action='append',choices=['aot','interpreter','jit'])
a=p.parse_args()
root=Path(__file__).resolve().parents[4]
validator=root/'docs/experiments/http-body-stream/validate.py'
spec=importlib.util.spec_from_file_location('stream_validator',validator)
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
original=json.loads(a.contracts.read_text());assert original['passed']
b=a.build.resolve()
commands={mode:[str(b/'app') if x==original['commands']['aot'][0] else str(b/'app.dll') if x.endswith('/app.dll') else x for x in command] for mode,command in original['commands'].items()}
context=next(c['command'][5:] for c in json.loads((b/'build.json').read_text())['commands'] if c['phase']=='native-compilation')
for mode in ['interpreter','jit']:
    prefix=commands[mode][:commands[mode].index('--system')]
    commands[mode]=prefix+(context[:context.index('--compile-system')] if mode=='interpreter' else context)
if a.mode:commands={mode:commands[mode] for mode in a.mode}
r=dict(passed=False,commands=commands,cases={},inputs={})
for path in [Path(__file__),validator,a.contracts,b/'app',b/'app.dll',b/'build.json']:
    r['inputs'][str(path.resolve())]=hashlib.sha256(path.read_bytes()).hexdigest()
try:
    for mode,command in commands.items():
        r['cases'][mode]=m.run_case(command,'server');print('PASS',mode,flush=True)
    r['passed']=True
finally:a.output.write_text(json.dumps(r,indent=2)+'\n')
