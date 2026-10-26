from pathlib import Path
import json,subprocess
root=Path('/Users/robert/Projects/neoclr');work=root/'target/specialization-lookup-20261026'
def run(command,name):
 print(name,flush=True)
 with open(work/(name+'.log'),'w') as f:
  r=subprocess.run(command,cwd=root,stdout=f,stderr=subprocess.STDOUT)
 assert r.returncode==0,(name,r.returncode)
run(['python3','/tmp/neoclr-specialization-prepare-pairs.py'],'prepare-pairs')
build=work/'paired';b=json.loads((build/'build.json').read_text());modes=list(b['recipes'])
for variant in ['baseline','indexed']:
 run([str(work/(variant+'-runner')),'--list'],variant+'-loader-qualification')
base=['python3','benchmarks/gc-suite/matrix/http.py','--build',str(build),'--modes',*modes,'--samples','json','--protocols','pipeline','--requests','4096']
run([*base,'--output',str(work/'paired-http'),'--rounds','3'],'paired-http')
run([*base,'--output',str(work/'paired-memory'),'--rounds','1','--process-memory'],'paired-memory')
run(['python3','benchmarks/gc-suite/matrix/profile-lazy-startup.py','--build',str(build),'--output',str(work/'startup-profile'),'--modes','baseline-lazy-live-jit','indexed-lazy-live-jit','--kind','cpu'],'startup-profile')
(work/'completed.json').write_text(json.dumps({'passed':True})+'\n')
