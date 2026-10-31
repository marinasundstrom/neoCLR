"""Private HttpServer-only eager/lazy and synchronous/concurrent GC screen."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import resource

ROOT = Path(__file__).resolve().parents[3]
p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--build', type=Path, required=True)
p.add_argument('--runner', type=Path, required=True)
p.add_argument('--sync-host', type=Path, required=True)
p.add_argument('--concurrent-host', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
p.add_argument('--repetitions', type=int, default=2)
a = p.parse_args()
out = a.output.resolve(); out.mkdir(parents=True, exist_ok=False)
spec = importlib.util.spec_from_file_location('benchmark', ROOT/'scripts/benchmark-neo-web.py')
m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
build = json.loads((a.build/'build.json').read_text())
original = next(c['command'] for c in build['commands'] if c['phase']=='native-compilation')
sha = lambda path: hashlib.sha256(Path(path).read_bytes()).hexdigest()
report = dict(scope='HttpServer-only, private qualified live-store runner, owned hosts, 32 MiB heap, fresh sequential connections; not a server-capacity test.',
              runnerSha256=sha(a.runner), appSha256=sha(a.build/'app.dll'),
              hosts={name: dict(path=str(path.resolve()), sha256=sha(path)) for name,path in [('sync',a.sync_host),('concurrent',a.concurrent_host)]}, cases=[])
launch = [str(a.runner.resolve()), 'mutation_store::tests::full_http_selection_compiles_with_live_gc', '--exact', '--ignored', '--nocapture']
failed = set()
for repetition in range(a.repetitions):
    modes = [('sync',False),('concurrent',False),('concurrent',True),('sync',True)]
    if repetition%2: modes.reverse()
    for collector,lazy in modes:
        if (collector,lazy) in failed: continue
        command = original.copy(); command[4] = report['hosts'][collector]['path']; command.insert(1,'--jit-run')
        path = out/f'{collector}-{int(lazy)}-selection.json'; path.write_text(json.dumps(command,indent=2)+'\n')
        env = dict(NEOCLR_LIVE_SELECTION_COMMAND=str(path), NEOCLR_LIVE_SELECTION_MODE='live' if collector=='concurrent' else 'sync', NEOCLR_LIVE_STACK_POLICY='qualified', NEOCLR_JIT_LAZY=str(int(lazy)))
        saved = {key:os.environ.get(key) for key in env}; os.environ.update(env)
        row = dict(collector=collector, lazy=lazy, repetition=repetition+1, command=launch, environment=env)
        try:
            before = resource.getrusage(resource.RUSAGE_CHILDREN)
            row.update(m.measure(launch,1000,20,'/',b'42',None))
            after = resource.getrusage(resource.RUSAGE_CHILDREN)
            row['compilerAndRequestsCpuSeconds'] = after.ru_utime+after.ru_stime-before.ru_utime-before.ru_stime
            row['passed'] = True
            print(collector,lazy,repetition+1,row['readySeconds'],row['firstRequestSeconds'],row['requestsPerSecond'],flush=True)
        except Exception as error:
            row.update(passed=False,error=str(error)); failed.add((collector,lazy))
            print(collector,lazy,'FAILED',str(error)[-1200:],flush=True)
        finally:
            for key,value in saved.items():
                if value is None: os.environ.pop(key,None)
                else: os.environ[key]=value
        report['cases'].append(row); (out/'results.json').write_text(json.dumps(report,indent=2)+'\n')
report['passed'] = all(c['passed'] for c in report['cases'])
(out/'results.json').write_text(json.dumps(report,indent=2)+'\n')
