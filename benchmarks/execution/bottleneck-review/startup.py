#!/usr/bin/env python3
"""Refresh checked ordinary-mode startup timings; capture profiles separately."""
import argparse
import gzip
import hashlib
import json
import resource
import subprocess
import time
from pathlib import Path

root = Path(__file__).resolve().parents[3]
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
out = a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
prior_path = root/'benchmarks/execution/gc-mode-comparison/ordinary.json.gz'
prior = json.loads(gzip.decompress(prior_path.read_bytes()))
r = dict(passed=False, revision=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),
         builds=[], runs=[], profiles=[], inputs={}, scope='Current binaries and native sources; retained qualified Raven assemblies. External 1 MiB paced native heap.')

def record(path):
    path = Path(path)
    if path.is_file():
        r['inputs'][str(path.resolve())] = hashlib.sha256(path.read_bytes()).hexdigest()

def save():
    (out/'results.json').write_text(json.dumps(r,indent=2)+'\n')

def run(cmd):
    for arg in cmd:
        record(arg)
    before = resource.getrusage(resource.RUSAGE_CHILDREN)
    start = time.perf_counter()
    x = subprocess.run(cmd,capture_output=True,text=True,timeout=180)
    after = resource.getrusage(resource.RUSAGE_CHILDREN)
    return dict(command=cmd,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr,
                wallSeconds=time.perf_counter()-start,
                cpuSeconds=after.ru_utime+after.ru_stime-before.ru_utime-before.ru_stime)

def build(cmd):
    row = run(cmd)
    r['builds'].append(row)
    save()
    assert row['exitCode']==0, row['stderr']
    if len(row['stdout'])>10000:
        row['stdoutSha256']=hashlib.sha256(row['stdout'].encode()).hexdigest()
        row['stdout']='(compiler inspection omitted; hash retained)'

try:
    record(__file__)
    record(prior_path)
    for path in (root/'runtime/native').rglob('*.[ch]'):
        record(path)
    host = out/'paced.dylib'
    cmd = next(x['command'].copy() for x in prior['commands'] if x['label']=='host-build' and '/paced-False.dylib' in ' '.join(x['command']))
    cmd[cmd.index('-o')+1] = str(host)
    build(cmd)
    configurations=[]
    for case in prior['cases']:
        sample=case['sample']
        command=next(x['command'].copy() for x in prior['commands'] if x['label']=='aot-compile' and f'/{sample}/bin/' in ' '.join(x['command']))
        old_object=command[4]
        command[0]=str(root/'target/release/neoclr-codegen')
        command[4]=str(out/(sample+'.o'))
        build(command)
        exe=out/(sample+'-aot')
        link=next(x['command'].copy() for x in prior['commands'] if x['label']=='aot-link' and f'/{sample}-paced-False' in ' '.join(x['command']))
        link=[command[4] if x==old_object else x for x in link]
        link[link.index('-o')+1]=str(exe)
        build(link)
        interpreter=next(x['command'].copy() for x in case['runs'] if x['mode']=='interpreter' and not x['diagnostic'])
        interpreter[0]=str(root/'target/release/neoclr')
        jit=[command[0],'--jit-managed','--closed-world',command[2],command[3],str(host),*command[5:]]
        for mode,cmd in [('interpreter',interpreter),('jit',jit),('aot',[str(exe)])]:
            configurations.append((sample,mode,cmd,case['expected']))
    # All compilation finishes before timings. Alternate forward/reverse order.
    for repetition in range(3):
        for sample,mode,cmd,expected in configurations[::(-1 if repetition%2 else 1)]:
            row=run(cmd)
            row.update(sample=sample,mode=mode,repetition=repetition)
            r['runs'].append(row)
            if mode=='jit':
                row['jit']=json.loads(next(x[11:] for x in row['stderr'].splitlines() if x.startswith('JIT_RESULT ')))
                assert row['jit']['guestExitCode']==0
            assert row['exitCode']==0 and row['stdout']==expected
            save()
            print(json.dumps({k:row[k] for k in ['sample','mode','wallSeconds']}),flush=True)
    for sample,mode,cmd,expected in configurations:
        if sample!='collections' or mode=='aot':
            continue
        proc=subprocess.Popen(cmd,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
        try:
            time.sleep(.15)
            target=out/(mode+'.sample.txt')
            capture=subprocess.run(['/usr/bin/sample',str(proc.pid),'1','1','-file',str(target)],capture_output=True,text=True,timeout=15)
            stdout,stderr=proc.communicate(timeout=120)
            r['profiles'].append(dict(mode=mode,command=cmd,exitCode=proc.returncode,stdout=stdout,stderr=stderr,
                                      sampleExitCode=capture.returncode,sampleStderr=capture.stderr,file=str(target)))
            assert proc.returncode==0 and stdout==expected and capture.returncode==0
        finally:
            if proc.poll() is None:
                proc.kill()
            proc.wait()
    r['passed']=True
finally:
    save()
