from pathlib import Path
import json,subprocess,hashlib,gzip
root=Path.cwd();w=root/'target/selection-box-scan-20261028'
assert json.loads((w/'completed.json').read_text())['passed']
subprocess.run(['python3','/tmp/neoclr-box-summarize-20261028.py'],check=True)
folder=w/'startup-profile'
for mode in ['baseline-lazy-live-jit','indexed-lazy-live-jit']:
 subprocess.run(['xcrun','xctrace','export','--input',str(folder/(mode+'-startup.trace')),'--xpath','/trace-toc/run[@number="1"]/data/table[@schema="cpu-profile"]','--output',str(folder/(mode+'.cpu.xml'))],check=True,capture_output=True)
subprocess.run(['python3','benchmarks/gc-suite/matrix/summarize-lazy-cpu.py','--input',str(folder/'baseline-lazy-live-jit.cpu.xml'),str(folder/'indexed-lazy-live-jit.cpu.xml'),'--output',str(w/'cpu-summary.json')],check=True)
m=json.loads((w/'paired-memory/results.json').read_text());assert m['passed'];rows=[]
for mode in ['eager-sync-jit','lazy-sync-jit','eager-live-jit','lazy-live-jit']:
 ts={v:next(t for t in m['trials'] if t['mode']==v+'-'+mode) for v in ['baseline','indexed']}
 row={'mode':mode,'nativeMemory':{v:t['jit']['nativeMemory'] for v,t in ts.items()},'processMemory':{v:t['processMemory'] for v,t in ts.items()}}
 rows.append(row)
(w/'memory-summary.json').write_text(json.dumps({'rows':rows},indent=2)+'\n')
out=root/'docs/validation/selection-box-scan-20261028'
files={'environment.json':w/'environment.json','summary.json':w/'summary.json','memory-summary.json':w/'memory-summary.json','cpu-summary.json.gz':w/'cpu-summary.json','build.json.gz':w/'paired/build.json','bootstrap.json.gz':w/'bootstrap.json','codegen-tests.txt.gz':Path('/tmp/neoclr-box-scan-tests.log'),'baseline-build.txt.gz':Path('/tmp/neoclr-box-scan-baseline-build.log'),'candidate-build.txt.gz':Path('/tmp/neoclr-box-scan-build.log'),'bootstrap.py':Path('/tmp/neoclr-box-bootstrap.py'),'measure.py':Path('/tmp/neoclr-box-measure-20261028.py'),'summarize.py':Path('/tmp/neoclr-box-summarize-20261028.py'),'finish.py':Path(__file__)}
for stage in ['smoke','paired-http','paired-memory','startup-profile']:files[stage+'-results.json.gz']=w/stage/'results.json'
for p in folder.glob('*.cpu.xml'):files[p.name+'.gz']=p
for p in [w/'baseline-selection.rs',w/'candidate-selection.rs',*list((w/'hosts').glob('*.c')),*list((w/'hosts').glob('*selection.json'))]:files[p.name+'.gz']=p
# Preserve the actual managed inputs for this cohort, independent of target cleanup.
for p in (w/'release/neoclr-native-poc/lib').glob('*'):
 if p.suffix in ['.dll','.neox'] or p.name=='bundle.json':files['library-'+p.name+'.gz']=p
for p in (w/'guest/bin/neoclr').glob('*'):files['guest-'+p.name+'.gz']=p
index={}
for name,src in files.items():
 data=src.read_bytes();dest=out/name;dest.write_bytes(gzip.compress(data,mtime=0) if name.endswith('.gz') else data);index[name]={'source':str(src),'sourceSha256':hashlib.sha256(data).hexdigest(),'archiveSha256':hashlib.sha256(dest.read_bytes()).hexdigest()}
patch=subprocess.check_output(['git','diff','--','runtime/codegen/src/selection.rs']);dest=out/'candidate.patch.gz';dest.write_bytes(gzip.compress(patch,mtime=0));index[dest.name]={'archiveSha256':hashlib.sha256(dest.read_bytes()).hexdigest()}
(out/'evidence-index.json').write_text(json.dumps(index,indent=2)+'\n')
assert (root/'runtime/codegen/src/selection.rs').read_bytes()==(w/'candidate-selection.rs').read_bytes()
print('Archived evidence')
