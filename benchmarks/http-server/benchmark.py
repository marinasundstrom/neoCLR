#!/usr/bin/env python3
"""Measure the same plain-text Raven HTTP server with a qualified macOS or Windows bundle."""
import argparse
import sys
import importlib.util
import json
import math
import shutil
import os
from pathlib import Path
import platform
import statistics
import runpy
import socket
import subprocess
import time
from xml.sax.saxutils import escape

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('http_workbench', ROOT/'benchmarks/jit-http/benchmark.py')
workbench = importlib.util.module_from_spec(spec)
spec.loader.exec_module(workbench)
sha = workbench.sha
from host import target, cpu_description, load_average, process_snapshot


def server_sample(command, workload):
    guest = workbench.Guest(command)
    try:
        port, ready = guest.line()
        samples = []
        for index in range(32):
            start = time.perf_counter()
            with socket.create_connection(('127.0.0.1', int(port)), timeout=60) as peer:
                peer.sendall(workbench.REQUEST)
                data = b''
                while True:
                    chunk = peer.recv(4096)
                    if not chunk:
                        break
                    data += chunk
                    if len(data) > 8192:
                        raise ValueError('Oversized response')
                headers, body = data.split(b'\r\n\r\n', 1)
                lines = headers.split(b'\r\n')
                fields = dict(line.lower().split(b': ', 1) for line in lines[1:])
                if lines[0] != b'HTTP/1.1 200 OK' or fields.get(b'connection') != b'close' or int(fields.get(b'content-length',b'-1')) != len(body):
                    raise ValueError('Invalid response framing/status')
                if workload == 'plain':
                    if body != workbench.BODY:
                        raise ValueError('Invalid text body')
                elif fields.get(b'content-type') != b'application/json; charset=utf-8' or json.loads(body) != {'id':42,'name':'Café team','active':True}:
                    raise ValueError('Invalid JSON body')
            if index >= 8:
                samples.append(time.perf_counter()-start)
        guest.line(b'DONE')
        guest.finish()
        return dict(startupSeconds=ready-guest.started, requestSeconds=samples,
                    measuredSeconds=sum(samples), requestsPerSecond=24/sum(samples))
    finally:
        guest.close()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bundle', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--rounds', type=int, default=5)
    parser.add_argument('--prepare-only', action='store_true', help='Build inputs without measuring; prepared is not a passing benchmark')
    parser.add_argument('--workload', choices=['plain','json','database'], default='plain')
    parser.add_argument('--server-source', type=Path, help='Explicit server fixture for syntax-compatible historical comparisons; its bytes are hashed')
    parser.add_argument('--postgres-port', type=int)
    parser.add_argument('--postgres-container')
    args = parser.parse_args()
    try:
        host_target = target()
    except ValueError as error:
        parser.error(str(error))
    if args.rounds < 5:
        parser.error('Requires at least five measured rounds')
    if args.workload == 'database' and host_target['http'] == 'windows-http':
        parser.error('The optional database benchmark shared-host linker is not yet supported on Windows')
    bundle = args.bundle.resolve()
    manifest = json.loads((bundle/'manifest.json').read_text())
    if not manifest.get('jit', {}).get('default'):
        parser.error('Requires a qualified default-JIT bundle')
    for name, digest in manifest['files'].items():
        if sha(bundle/name) != digest:
            raise ValueError('Bundle drift: '+name)
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    project = out/'project'
    project.mkdir()
    source = args.server_source.resolve() if args.server_source else ROOT/('benchmarks/jit-http/Server.rvn' if args.workload=='plain' else 'benchmarks/http-server/'+('JsonServer.rvn' if args.workload=='json' else 'DatabaseServer.rvn'))
    (project/'Main.rvn').write_bytes(source.read_bytes())
    if args.workload != 'plain':
        (project/'Model.rvn').write_bytes((ROOT/'benchmarks/http-server/Model.rvn').read_bytes())
    if args.workload == 'database':
        if not args.postgres_port or not args.postgres_container:
            parser.error('Database workload requires --postgres-port and --postgres-container')
        (project/'Configuration.rvn').write_text(f'class DatabaseConfiguration {{ static val Port: int => {args.postgres_port} }}\n')
    (project/'App.rvnproj').write_text('''<Project Sdk="Microsoft.NET.Sdk">
  <Import Project="%s" />
  <PropertyGroup>
    <TargetFramework>net10.0</TargetFramework>
    <OutputType>Exe</OutputType>
    <AssemblyName>App</AssemblyName>
  </PropertyGroup>
</Project>
''' % escape(str(bundle/'lib/NeoCLR.ClassLibrary.props'), {'"':'&quot;'}))
    report = dict(passed=False, revision=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),
                  platform=platform.platform(), cpu=cpu_description(), hostTarget=host_target,
                  date=time.strftime('%Y-%m-%d'), rounds=args.rounds,
                  workload=args.workload, scope='GET /greeting; HTTP/1.1 loopback; connection close; concurrency 1; Python load generator on same machine',
                  requestsPerProcess=32, discardedRequestsPerProcess=8,
                  wholeProcessWarmupsPerMode=1, inputHashes={}, commands={}, samples=[], warmups=[], summary={},
                  measurement='Client wall time from connect through validated response and EOF. Build excluded; process-to-port startup reported separately, including JIT compilation.',
                  limitations=['Short fixed-count baseline, not saturation or sustained throughput.',
                               'p99 is a nearest-rank estimate from 120 measured requests per mode at five rounds.',
                               'No RSS/CPU profile, HTTPS, keep-alive or cross-runtime comparison.'])
    inputs=[*project.glob('*.rvn'), Path(__file__).resolve(), ROOT/'benchmarks/jit-http/benchmark.py', ROOT/'benchmarks/framework/process.py',ROOT/'benchmarks/framework/host.py',source,bundle/'manifest.json']
    report['inputHashes']={str(p):sha(p) for p in inputs}
    try:
        native=out/'native'
        roots=[]
        if args.workload != 'plain':
            def run(command):
                return subprocess.run(list(map(str,command)),check=True,capture_output=True,text=True)
            run(['dotnet',bundle/'sdk/tools/rvnc/rvnc.dll','neoclr','--project',project/'App.rvnproj'])
            prepare=runpy.run_path(str(bundle/'tools/run-native-sample.py'))['reflection_roots']
            roots=list(map(str,prepare(bundle/manifest['runtime'],project/'bin/neoclr/App.dll',['Person'],run)))
        if args.workload == 'database':
            report['database']={
                'container':args.postgres_container,'port':args.postgres_port,
                'configuration':subprocess.check_output(['docker','inspect',args.postgres_container,'--format','{{.Config.Image}} {{.Image}} CPUs={{.HostConfig.NanoCpus}} Memory={{.HostConfig.Memory}}'],text=True).strip(),
                'version':subprocess.check_output(['docker','exec',args.postgres_container,'psql','-U','postgres','-Atc','SELECT version()'],text=True).strip(),
                'query':'SELECT $1::int AS id, $2::text AS name, $3::bool AS active',
                'parameters':[42,'Café team',True], 'rows':1,'connection':'One reused SQL connection per server process, opened before readiness; no tables or persistent dataset.'}

        build=[sys.executable,str(ROOT/'scripts/build-native-project.py'),'--profile',host_target['http'],'--project',str(project/'App.rvnproj'),
               '--bundle',str(bundle),'--aot',str(bundle/manifest['jit']['compiler']),'--output',str(native),
               '--http-heap-bytes',str(manifest['jit']['heapBytes']),'--byte-array-limit',str(manifest['jit']['byteArrayLimit']),
               '--command-timeout','600',*roots]
        if args.workload == 'database':
            # PostgreSQL's released managed library requires runtime-async bindings.
            # Reuse the qualified shared host, as in sql-postgresql/validate.py.
            native.mkdir()
            shutil.copyfile(project/'bin/neoclr/App.dll',native/'app.dll')
            context=['--system',str(bundle/manifest['runtimeSeed']),'--object-root',str(bundle/manifest['objectRoot'])]
            for module in manifest['modules']:
                context += ['--module',str(bundle/module)]
            flags=['--compile-system','--bind-user-fault','--reference-arena','--native-gc',
                   '--bind-console-write-line','--bind-utf8-text','--bind-int32-to-string','--native-stack-budget',
                   '--bind-integer-text','--bind-character-text','--bind-task-queue','--bind-scheduler','--bind-timers',
                   '--bind-socket-listener','--bind-socket-accept','--bind-socket-transfer','--bind-socket-client','--bind-runtime-async',
                   '--bind-cryptography','--bind-tls','--bind-system-culture','--bind-time-zone-rules']
            wrapper=native/'main.c'
            wrapper.write_text('#include "native-gc.h"\nextern int32_t neoclr_entry_v4(int32_t,int32_t*,neoclr_aot_context*);\nextern int32_t neoclr_jit_run_v1(const void*,int32_t,int32_t,const char**);\nint main(int argc,char**argv) { return neoclr_jit_run_v1((const void*)neoclr_entry_v4,0,argc,(const char**)argv); }\n')
            host=bundle/manifest['jit']['host']
            commands=[[str(bundle/manifest['jit']['compiler']),'--closed-world',str(native/'app.dll'),'@entry',str(native/'app.o'),*context,*flags,*roots],
                      [os.environ.get('CC','cc'),'-O2','-I',str(ROOT/'docs/experiments/aot-console'),str(wrapper),str(native/'app.o'),str(host),'-Wl,-rpath,'+str(host.parent),'-o',str(native/'app')]]
            report['build']={'profile':'aot-shared-runtime-async-host','commands':[], 'httpHeapBytes':manifest['jit']['heapBytes']}
            for command in commands:
                started=time.perf_counter()
                result=subprocess.run(command,capture_output=True,text=True,timeout=600)
                report['build']['commands'].append(dict(command=command,exitCode=result.returncode,elapsedSeconds=time.perf_counter()-started,stdout=result.stdout,stderr=result.stderr))
                if result.returncode:
                    raise RuntimeError(result.stdout+result.stderr)
            report['build']['artifacts']={n:sha(native/n) for n in ['app.dll','app.o','app','main.c']}
            report['inputHashes'][str(ROOT/'docs/experiments/aot-console/native-gc.h')]=sha(ROOT/'docs/experiments/aot-console/native-gc.h')
            report['aotDeployment']='Ahead-of-time guest code linked to the released shared native host; not a standalone SQL executable.'
        else:
            with (out/'build.log').open('w') as log:
                subprocess.run(build,stdout=log,stderr=subprocess.STDOUT,check=True)
            report['buildCommand']=build
            report['build']=json.loads((native/'build.json').read_text())
        assembly=native/'app.dll'
        runtime=str(bundle/manifest['runtime'])
        context=['--system',str(bundle/manifest['runtimeSeed']),'--object-root',str(bundle/manifest['objectRoot'])]
        for module in manifest['modules']:
            context += ['--module',str(bundle/module)]
        report['commands']={'interpreter':[runtime,'run',str(assembly),'--execution','interpreter','--instructions','100000000',*context],
                            'jit':[runtime,'run',str(assembly),*roots],'aot':[str(native/host_target['executable'])]}
        report['bundleManifest']=manifest
        if args.prepare_only:
            report['prepared']=True
            return
        report['loadBefore']=load_average()
        report['hostProcessesBefore']=process_snapshot()
        modes=['interpreter','jit','aot']
        for index in range(-1,args.rounds):
            offset=max(index,0)%3
            for mode in modes[offset:]+modes[:offset]:
                row=server_sample(report['commands'][mode],args.workload)
                row.update(mode=mode,round=index)
                report['warmups' if index<0 else 'samples'].append(row)
                (out/'results.json').write_text(json.dumps(report,indent=2)+'\n')
                print(f'{mode} round {index}: {row["requestsPerSecond"]:.2f} requests/s',flush=True)
        report['loadAfter']=load_average()
        for mode in modes:
            rows=[r for r in report['samples'] if r['mode']==mode]
            rates=[r['requestsPerSecond'] for r in rows]
            starts=[r['startupSeconds'] for r in rows]
            latencies=sorted(t for r in rows for t in r['requestSeconds'])
            report['summary'][mode]={'requestsPerSecond':{'median':statistics.median(rates),'min':min(rates),'max':max(rates)},
              'startupSecondsMedian':statistics.median(starts),
              'latencySeconds':{f'p{q}':latencies[math.ceil(q/100*len(latencies))-1] for q in [50,95,99]},
              'validatedRequests':32*(args.rounds+1),'measuredRequests':len(latencies),'errors':0}
        for path,digest in report['inputHashes'].items():
            if sha(path)!=digest:
                raise ValueError('Input changed during measurement: '+path)
        for name,digest in manifest['files'].items():
            if sha(bundle/name)!=digest:
                raise ValueError('Bundle changed during measurement: '+name)
        report['passed']=True
    finally:
        (out/'results.json').write_text(json.dumps(report,indent=2)+'\n')


if __name__=='__main__':
    main()
