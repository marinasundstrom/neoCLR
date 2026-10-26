"""Prepare or measure READY-instrumented compute/allocation samples for lazy JIT.

READY is the first statement of managed Main. Median process-to-READY includes
metadata, lowering, initial code generation and first-call initialization.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import queue
import resource
import statistics
import subprocess
import threading
import time

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--phase', choices=['prepare','measure'], required=True)
p.add_argument('--build', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
p.add_argument('--source-build', type=Path)
p.add_argument('--compiler', type=Path)
p.add_argument('--rounds', type=int, default=5)
a = p.parse_args()
root = Path(__file__).resolve().parents[3]
out = a.output.resolve()
out.mkdir(parents=True, exist_ok=False)
source = a.build.resolve()/'build.json'
build = json.loads(source.read_text())
assert build['passed']
for path, digest in build['inputs'].items():
    assert hashlib.sha256(Path(path).read_bytes()).hexdigest() == digest, ('stale input',path)
def digest(path): return hashlib.sha256(Path(path).read_bytes()).hexdigest()

if a.phase == 'prepare':
    original = json.loads((a.source_build/'results.json').read_text())
    assert original['passed'] and original['iterations'] == 1
    r = dict(passed=False, scope=__doc__, recipes={}, inputs=dict(build['inputs']), commands=[])
    for name in ['compute','steady']:
        case = next(c for c in original['cases'] if c['sample'] == name)
        old_source = a.source_build.resolve()/name/(name+'.rvn')
        old_project = a.source_build.resolve()/name/'App.rvnproj'
        folder = out/name
        folder.mkdir()
        text = old_source.read_text()
        marker = 'async func Main() -> System.Concurrency.Tasks.Task<()> {\n'
        assert text.count(marker) == 1
        new_source = folder/(name+'.rvn')
        new_source.write_text(text.replace(marker, marker+'    Console.WriteLine("READY")\n'))
        project = folder/'App.rvnproj'
        project.write_text(old_project.read_text().replace(str(old_source), str(new_source)))
        command = ['dotnet', str(a.compiler.resolve()), 'neoclr', '--project', str(project)]
        x = subprocess.run(command, capture_output=True, text=True, timeout=180)
        r['commands'].append(dict(command=command,exitCode=x.returncode,stdout=x.stdout,stderr=x.stderr))
        x.check_returncode()
        app = folder/'bin/neoclr/App.dll'
        for mode, recipe in build['recipes'].items():
            native = next(n for n in case['native'] if n['mode'] == recipe['collector'])
            args = native['selectionCommand']
            http = json.loads(Path(recipe['env']['NEOCLR_LIVE_SELECTION_COMMAND']).read_text())
            host = next(v for v in http if v.endswith('.dylib'))
            selected = [args[0], '--jit-managed', args[1], str(app), args[3], host, *args[5:]]
            path = folder/(mode+'.json')
            path.write_text(json.dumps(selected,indent=2)+'\n')
            env = dict(recipe['env'], NEOCLR_LIVE_SELECTION_COMMAND=str(path))
            r['recipes'][name+'-'+mode] = dict(recipe, sample=name, expected=case['expectedPerIteration'],env=env)
            r['inputs'][str(path)] = digest(path)
        for path in [old_source,old_project,new_source,project,app]: r['inputs'][str(path)] = digest(path)
    for path in [source,a.compiler.resolve(),Path(__file__).resolve()]:r['inputs'][str(path)] = digest(path)
    r['passed'] = True
    (out/'build.json').write_text(json.dumps(r,indent=2)+'\n')
else:
    assert a.rounds >= 3
    r = dict(passed=False,scope=__doc__,rounds=a.rounds,inputs={str(source):digest(source)},trials=[],rows=[])
    try:
        for repetition in range(a.rounds):
            keys = list(build['recipes'])
            if repetition % 2: keys.reverse()
            for key in keys:
                recipe = build['recipes'][key]
                print(repetition,key,flush=True)
                env = {k:v for k,v in os.environ.items() if not k.startswith('NEOCLR_')}
                env.update(recipe['env'])
                before = resource.getrusage(resource.RUSAGE_CHILDREN)
                started = time.perf_counter()
                process = subprocess.Popen(recipe['command'],cwd=root,env=env,text=True,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
                stdout=[];stderr=[];events=queue.Queue()
                def drain(stream, output, notify=False):
                    for line in stream:
                        output.append(line)
                        if notify and line.strip()=='READY': events.put(time.perf_counter()-started)
                    if notify: events.put(None)
                readers=[threading.Thread(target=drain,args=(process.stdout,stdout,True)),threading.Thread(target=drain,args=(process.stderr,stderr))]
                for reader in readers:reader.start()
                try:
                    ready=events.get(timeout=90)
                    assert ready is not None,'missing READY'
                    status=process.wait(timeout=180)
                finally:
                    if process.poll() is None:process.kill();process.wait()
                    for reader in readers:reader.join()
                elapsed=time.perf_counter()-started
                after=resource.getrusage(resource.RUSAGE_CHILDREN)
                output=''.join(stdout);errors=''.join(stderr)
                assert status==0,errors
                assert recipe['expected'] in output,output
                profile=next(line.split()[1:] for line in stderr if line.startswith('PROFILE '))
                jit=json.loads(next(line[len('JIT_RESULT '):] for line in stderr if line.startswith('JIT_RESULT ')))
                assert jit['guestExitCode']==0
                row=dict(mode=key,sample=recipe['sample'],collector=recipe['collector'],round=repetition,readySeconds=ready,
                    wholeProcessWallSeconds=elapsed,wholeProcessCpuSeconds=after.ru_utime+after.ru_stime-before.ru_utime-before.ru_stime,
                    invocationCpuSeconds=float(profile[0]),invocationWallSeconds=float(profile[1]),allocations=int(profile[2]),
                    allocatedBytes=int(profile[3]),collections=int(profile[4]),jit=jit,stdout=output,stderr=errors,passed=True)
                if recipe['collector']=='live':
                    counts=next(list(map(int,l.split()[1:])) for l in stderr if l.startswith('CANDIDATE '))
                    assert counts[0]==counts[1]+counts[2] and counts[5]==0
                    row['candidateCounts']=counts
                r['trials'].append(row)
                (out/'progress.json').write_text(json.dumps(r,indent=2)+'\n')
        for key in build['recipes']:
            trials=[t for t in r['trials'] if t['mode']==key]
            r['rows'].append(dict(mode=key,**{k:statistics.median(t[k] for t in trials) for k in ['readySeconds','wholeProcessWallSeconds','wholeProcessCpuSeconds','invocationCpuSeconds','allocations','allocatedBytes','collections']}))
        for name in ['compute','steady']:
            for mode in ['sync','live']:
                trials=[t for t in r['trials'] if t['sample']==name and t['collector']==mode]
                assert len({(t['allocations'],t['allocatedBytes']) for t in trials})==1,'allocation work mismatch'
        r['passed']=True
    finally:(out/'results.json').write_text(json.dumps(r,indent=2)+'\n')
