from pathlib import Path
import time,json,subprocess
w=Path.cwd()/'target/interface-views-20261028'
start=time.monotonic()
while not (w/'completed.json').exists():
 if time.monotonic()-start>900:raise TimeoutError('Primary measurement queue did not complete')
 time.sleep(1)
assert json.loads((w/'completed.json').read_text())['passed']
with open(w/'long-eager-sync.log','w') as f:
 r=subprocess.run(json.loads((w/'long-command.json').read_text()),stdout=f,stderr=subprocess.STDOUT)
assert r.returncode==0,r.returncode
print('Long eager/sync comparison passed')
