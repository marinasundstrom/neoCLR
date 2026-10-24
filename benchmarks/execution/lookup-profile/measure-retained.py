#!/usr/bin/env python3
"""Compare cleared/retained exact-base caches without diagnostic instrumentation."""
import hashlib
import json
import os
from pathlib import Path
import queue
import socket
import subprocess
import threading
import time

root = Path(__file__).resolve().parents[3]
out = root/'target/lookup-profile/retained-capacity'
out.mkdir(parents=True, exist_ok=False)
prior = json.loads((root/'target/native-host-ownership/results.json').read_text())
client = root/'target/primitive-comparison-refresh/client'
r = dict(passed=False, builds=[], trials=[], inputs={})


def record(path):
    path = Path(path)
    r['inputs'][str(path)] = hashlib.sha256(path.read_bytes()).hexdigest()


def build(command, policy=None):
    env = dict(os.environ)
    env.pop('NEOCLR_CALL_EXPERIMENT', None)
    if policy is not None:
        env['NEOCLR_CALL_EXPERIMENT'] = policy
    x = subprocess.run(command, env=env, capture_output=True, text=True, timeout=120)
    r['builds'].append(dict(command=command, policy=policy, exitCode=x.returncode,
                            stdoutSha256=hashlib.sha256(x.stdout.encode()).hexdigest(), stderr=x.stderr))
    (out/'results.json').write_text(json.dumps(r, indent=2)+'\n')
    x.check_returncode()


def cpu(pid):
    raw = subprocess.check_output(['ps','-p',str(pid),'-o','time='], text=True).strip()
    value = 0.0
    for part in raw.split(':'):
        value = value*60+float(part)
    return raw, value


try:
    configs = []
    for mode in ['aot','jit']:
        old=root/('target/native-host-ownership/owned-aot' if mode=='aot' else 'target/native-host-ownership/owned.dylib')
        for buckets in ['cleared','retained']:
            label=f'{mode}-{buckets}'
            artifact=out/(label+('.dylib' if mode=='jit' else ''))
            command=next(x['command'].copy() for x in prior['commands'] if str(old) in x['command'])
            command[command.index('-o')+1]=str(artifact)
            if buckets=='cleared':
                command=[str(root/'target/lookup-profile/baseline-gc.c') if x.endswith('/native-gc.c') else x for x in command]
            build(command)
            record(artifact)
            invocation=next(x['command'].copy() for x in prior['trials'] if x['label']==f'{mode}-owned')
            invocation=[str(artifact) if x==str(old) else x for x in invocation]
            configs.append((label,invocation))
    for label, command in configs+configs[::-1]:
        proc = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        lines, errors, events = [], [], queue.Queue()
        row = dict(label=label, command=command)
        r['trials'].append(row)
        def read(stream, target):
            for line in stream:
                target.append(line)
                events.put(line)
        readers = [threading.Thread(target=read,args=(proc.stdout,lines)), threading.Thread(target=read,args=(proc.stderr,errors))]
        for reader in readers:
            reader.start()
        try:
            while True:
                line = events.get(timeout=60)
                if line.startswith('READY '):
                    break
            _, port, control = line.split()
            before_raw, before = cpu(proc.pid)
            def load(seconds):
                x = subprocess.run([str(client),port,str(seconds)], env=dict(os.environ,GOMAXPROCS='2'), capture_output=True,text=True,timeout=45)
                x.check_returncode()
                return json.loads(x.stdout)
            warmup, trial = load(1), load(5)
            after_raw, after = cpu(proc.pid)
            with socket.create_connection(('127.0.0.1',int(control)),timeout=5):
                pass
            proc.wait(timeout=30)
            for reader in readers:
                reader.join(timeout=2)
            assert proc.returncode == 0
            if label.startswith('jit-'):
                assert json.loads(next(line[11:] for line in errors if line.startswith('JIT_RESULT ')))['guestExitCode']==0
            count = sum(x['completed']+x['drained'] for x in [warmup,trial])
            row.update(warmup=warmup,trial=trial,psCpuBefore=before_raw,psCpuAfter=after_raw,
                       cpuSeconds=after-before,cpuMicrosecondsPerResponse=(after-before)*1e6/count)
            print(json.dumps(dict(label=label,requestsPerSecond=trial['requestsPerSecond'],cpuMicrosecondsPerResponse=row['cpuMicrosecondsPerResponse'])),flush=True)
        finally:
            if proc.poll() is None:
                proc.kill()
            proc.wait()
            for reader in readers:
                reader.join(timeout=2)
            row.update(exitCode=proc.returncode,stdout=lines,stderr=errors)
            (out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
    for path in [Path(__file__),client,root/'target/release/neoclr-codegen',root/'target/root-map/candidate.o',root/'target/lookup-profile/baseline-gc.c',root/'runtime/native/gc/native-gc.c',root/'runtime/native/gc/native-gc.h']:
        record(path)
    r['passed'] = True
finally:
    (out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
