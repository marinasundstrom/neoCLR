#!/usr/bin/env python3
"""Profile the checked candidate separately after uninstrumented capacity runs."""
import argparse,json,subprocess
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__);p.add_argument('--directory',type=Path,required=True);a=p.parse_args()
root=Path(__file__).resolve().parents[3];d=a.directory.resolve()
r=json.loads((d/'capacity/measurement.json').read_text());assert r['passed']
binary=str(d/'capacity/candidate-w4')
command=next(x['command'] for x in r['commands'] if '-o' in x['command'] and binary in x['command'])
report=dict(passed=True,workers=4,commands=[dict(command=command)],inputs=r['inputs'])
path=d/'profile-build.json';path.write_text(json.dumps(report,indent=2)+'\n')
commands=[
 ['python3',str(root/'benchmarks/http-server/primitive-comparison/stack-profile/capture.py'),'--build-report',str(path),'--client',str(root/'target/primitive-comparison-refresh/client'),'--output',str(d/'profile')],
 ['python3',str(root/'benchmarks/http-server/primitive-comparison/stack-profile/analyze.py'),'--capture',str(d/'profile/capture.json'),'--build-report',str(path),'--compilation',str(root/'target/slot-maps/build.json'),'--object',str(root/'target/slot-maps/plain.o'),'--output',str(d/'profile-analysis')]]
results=[]
for cmd in commands:
 x=subprocess.run(cmd,capture_output=True,text=True,timeout=120)
 results.append(dict(command=cmd,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr))
 (d/'profile-commands.json').write_text(json.dumps(results,indent=2)+'\n');x.check_returncode()
print('Checked candidate profile and exact object symbol mapping passed')
