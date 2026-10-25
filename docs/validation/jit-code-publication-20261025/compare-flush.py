import subprocess,json,os,hashlib
from pathlib import Path
runner=Path('target/release/deps/neoclr_codegen-12f33f26d2ff4df0')
rows=[]
for i in range(100):
 for flush in [False,True]:
  env=dict(os.environ)
  if flush:env['NEOCLR_TEST_FLUSH_TRANSFER_CODE']='1'
  else:env.pop('NEOCLR_TEST_FLUSH_TRANSFER_CODE',None)
  x=subprocess.run([str(runner),'mutation_store::tests::unambiguous_stack_transfers_elide_only_proved_stores','--exact','--nocapture'],env=env,capture_output=True,text=True)
  rows.append(dict(flush=flush,iteration=i,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr))
for flush in [False,True]:print(flush,sum(r['exitCode']==0 for r in rows if r['flush']==flush),'/100 pass')
Path('target/gc-stack-transfer-investigation/flush.json').write_text(json.dumps(dict(runnerSha256=hashlib.sha256(runner.read_bytes()).hexdigest(),trials=rows),indent=2)+'\n')
