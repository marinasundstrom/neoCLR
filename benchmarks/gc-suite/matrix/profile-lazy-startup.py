"""Profile process startup and first listen, with no HTTP traffic; diagnostics only.

The server is shut down through its control socket after READY and profiling.
Profiler activity perturbs startup; these times are never benchmark medians.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import queue
import socket
import subprocess
import threading
import time
import traceback

p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--build',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
p.add_argument('--modes',nargs='+',required=True)
p.add_argument('--kind',choices=['stacks','cpu'],default='stacks')
a=p.parse_args()
source=a.build.resolve()/'build.json';build=json.loads(source.read_text())
assert build['passed']
for path,digest in build['inputs'].items():
    assert hashlib.sha256(Path(path).read_bytes()).hexdigest()==digest,path
out=a.output.resolve();out.mkdir(parents=True,exist_ok=False)
r=dict(passed=False,diagnosticOnly=__doc__,kind=a.kind,trials=[],inputs={
    str(path.resolve()):hashlib.sha256(path.read_bytes()).hexdigest() for path in [source,Path(__file__)]})
try:
    for mode in a.modes:
        print(mode,flush=True)
        recipe=build['recipes'][mode]
        env={k:v for k,v in os.environ.items() if not k.startswith('NEOCLR_')};env.update(recipe['env'])
        row=dict(mode=mode,passed=False,command=recipe['command'],environment=recipe['env'])
        r['trials'].append(row)
        process=subprocess.Popen(recipe['command'],env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True,bufsize=1)
        output=[];errors=[];lines=queue.Queue()
        def collect(stream,dest,notify=False):
            for line in stream:
                dest.append(line)
                if notify:lines.put(line)
        threads=[threading.Thread(target=collect,args=(process.stdout,output,True)),
                 threading.Thread(target=collect,args=(process.stderr,errors))]
        for thread in threads:thread.start()
        profiler=None
        try:
            path=out/(mode+('-startup.txt' if a.kind=='stacks' else '-startup.trace'))
            cmd=(['sample',str(process.pid),'3','1','-file',str(path)] if a.kind=='stacks' else
                 ['xcrun','xctrace','record','--template','CPU Profiler','--attach',str(process.pid),'--time-limit','10s','--output',str(path)])
            row['profile']=dict(command=cmd,path=str(path))
            profiler=subprocess.Popen(cmd,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
            deadline=time.monotonic()+120
            while lines.get(timeout=max(.01,deadline-time.monotonic())).strip()!='READY':pass
            port=int(lines.get(timeout=10));control=int(lines.get(timeout=10))
            stdout,stderr=profiler.communicate(timeout=60)
            row['profile'].update(exitCode=profiler.returncode,stdout=stdout,stderr=stderr)
            with socket.create_connection(('127.0.0.1',control),timeout=10):pass
            process.wait(timeout=45)
            for thread in threads:thread.join(timeout=5)
            assert process.returncode==0
            assert 'DONE 0 0\n' in output
            row['jit']=json.loads(next(line[11:] for line in errors if line.startswith('JIT_RESULT ')))
            assert row['jit']['guestExitCode']==0
            assert profiler.returncode==0 and path.exists(),row['profile']
            if a.kind=='stacks':
                assert 'Call graph:' in path.read_text()
                row['profile']['sha256']=hashlib.sha256(path.read_bytes()).hexdigest()
            else:
                row['profile']['files']={str(f.relative_to(path)):hashlib.sha256(f.read_bytes()).hexdigest() for f in path.rglob('*') if f.is_file()}
            row['passed']=True
        except Exception:
            row['failure']=traceback.format_exc()
        finally:
            if profiler is not None and profiler.poll() is None:profiler.kill();profiler.communicate(timeout=10)
            if process.poll() is None:process.kill();process.wait(timeout=10)
            for thread in threads:thread.join(timeout=5)
            row.update(exitCode=process.returncode,stdout=''.join(output),stderr=''.join(errors))
        (out/'progress.json').write_text(json.dumps(r,indent=2)+'\n')
        assert row['passed'],row.get('failure')
    r['passed']=True
finally:
    (out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
