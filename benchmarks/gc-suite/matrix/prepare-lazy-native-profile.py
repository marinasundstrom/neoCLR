"""Pair lazy/eager compiler controls with explicitly instrumented native GC hosts.

The native-profile manifest supplies the diagnostic hosts; guest and compiler
inputs come from the lazy build. These runs cannot establish timing improvements.
"""
import argparse
import hashlib
import json
from pathlib import Path

p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--build',type=Path,required=True)
p.add_argument('--native-profile',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
a=p.parse_args()
source=a.build.resolve()/'build.json'
native=a.native_profile.resolve()/'build.json'
b=json.loads(source.read_text());n=json.loads(native.read_text())
assert b['passed'] and n['passed'] and n['diagnosticOnly']
for report in [b,n]:
    for path,digest in report['inputs'].items():
        assert hashlib.sha256(Path(path).read_bytes()).hexdigest()==digest,path
out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
r=dict(b,scope=__doc__,recipes={},inputs=dict(b['inputs'],**n['inputs']))
for key in ['diagnosticOnly','fiberSwitchProfile','fiberPhaseProfile','schedulerPhaseProfile']:
    if key in n:r[key]=n[key]
for mode,recipe in b['recipes'].items():
    collector=recipe['collector']
    profiled=n['recipes'][collector+'-jit']
    selection=Path(profiled['env']['NEOCLR_LIVE_SELECTION_COMMAND'])
    # Ensure changing the native host cannot silently change guest selection.
    old=json.loads(Path(recipe['env']['NEOCLR_LIVE_SELECTION_COMMAND']).read_text())
    new=json.loads(selection.read_text())
    assert [v for v in old if not v.endswith('.dylib')]==[v for v in new if not v.endswith('.dylib')]
    r['recipes'][mode]=dict(recipe,env=dict(recipe['env'],NEOCLR_LIVE_SELECTION_COMMAND=str(selection)))
for path in [source,native,Path(__file__).resolve()]:r['inputs'][str(path)]=hashlib.sha256(path.read_bytes()).hexdigest()
(out/'build.json').write_text(json.dumps(r,indent=2)+'\n')
