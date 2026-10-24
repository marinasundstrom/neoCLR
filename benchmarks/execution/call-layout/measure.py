#!/usr/bin/env python3
"""Isolate call lowering and helper linking with matched owned-heap HTTP hosts."""
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
out = root/'target/call-layout/results'
out.mkdir(parents=True, exist_ok=False)
prior = json.loads((root/'target/native-host-ownership/results.json').read_text())
recipe = json.loads((root/'target/root-map/build.json').read_text())['commands'][0]['command']
compiler = root/'target/call-layout/experiment-codegen'
helper = root/'target/native-host-ownership/owned.dylib'
client = root/'target/primitive-comparison-refresh/client'
wrapper = root/'target/gc-mode-comparison-final/main.c'
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
    link = next(x['command'].copy() for x in prior['commands'] if str(root/'target/native-host-ownership/owned-aot') in x['command'])
    # Non-PIC AOT was rejected by the Mach-O linker; retained separately.
    for policy in ['', 'long-range']:
        label = policy or 'baseline'
        obj = out/(label+'.o')
        command = recipe.copy()
        command[0], command[4] = str(compiler), str(obj)
        build(command, policy)
        record(obj)
        if not policy:
            assert obj.read_bytes() == Path(recipe[4]).read_bytes()
        exe = out/label
        command = [str(obj) if arg==recipe[4] else arg for arg in link]
        command[command.index('-o')+1] = str(exe)
        build(command)
        configs.append((label, [str(exe)]))
        record(exe)
    exe = out/'dynamic'
    command = link[:link.index('-std=c11')] + ['-std=c11','-O2',str(wrapper),str(out/'baseline.o'),str(helper),'-o',str(exe)]
    build(command)
    configs.append(('dynamic', [str(exe)]))
    record(exe)
    configs.append(('jit', next(x['command'] for x in prior['trials'] if x['label']=='jit-owned')))
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
            if label=='jit':
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
    for path in [compiler, helper, client, wrapper, Path(__file__), root/'target/call-layout/experiment.patch', root/'target/call-layout/baseline-codegen', root/'target/native-host-ownership/owned-host.c', *[Path(arg) for arg in recipe[2:] if Path(arg).is_file()], *(root/'runtime/native').rglob('*.[ch]')]:
        record(path)
    r['passed'] = True
finally:
    (out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
