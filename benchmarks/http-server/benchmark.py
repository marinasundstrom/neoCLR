#!/usr/bin/env python3
"""Measure the same plain-text Raven HTTP server with a qualified macOS bundle."""
import argparse
import importlib.util
import json
import math
import os
from pathlib import Path
import platform
import statistics
import subprocess
import time
from xml.sax.saxutils import escape

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('http_workbench', ROOT/'benchmarks/jit-http/benchmark.py')
workbench = importlib.util.module_from_spec(spec)
spec.loader.exec_module(workbench)
sha = workbench.sha


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bundle', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--rounds', type=int, default=5)
    args = parser.parse_args()
    if platform.system() != 'Darwin' or platform.machine() != 'arm64' or args.rounds < 5:
        parser.error('Requires macOS ARM64 and at least five measured rounds')
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
    source = ROOT/'benchmarks/jit-http/Server.rvn'
    (project/'Main.rvn').write_bytes(source.read_bytes())
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
                  platform=platform.platform(), cpu=subprocess.check_output(['sysctl','-n','machdep.cpu.brand_string'],text=True).strip(),
                  date=time.strftime('%Y-%m-%d'), rounds=args.rounds,
                  scope='Plain-text GET /greeting; 10 UTF-8 bytes; HTTP/1.1 loopback; connection close; concurrency 1; Python load generator on same machine',
                  requestsPerProcess=32, discardedRequestsPerProcess=8,
                  wholeProcessWarmupsPerMode=1, inputHashes={}, commands={}, samples=[], warmups=[], summary={},
                  measurement='Client wall time from connect through validated response and EOF. Build excluded; process-to-port startup reported separately, including JIT compilation.',
                  limitations=['Short fixed-count baseline, not saturation or sustained throughput.',
                               'p99 is a nearest-rank estimate from 120 measured requests per mode at five rounds.',
                               'No RSS/CPU profile, HTTPS, keep-alive, JSON, database or cross-runtime comparison.'])
    inputs=[Path(__file__).resolve(), ROOT/'benchmarks/jit-http/benchmark.py', ROOT/'benchmarks/framework/process.py',source,bundle/'manifest.json']
    report['inputHashes']={str(p):sha(p) for p in inputs}
    try:
        native=out/'native'
        build=['python3',str(ROOT/'scripts/build-native-project.py'),'--profile','http','--project',str(project/'App.rvnproj'),
               '--bundle',str(bundle),'--aot',str(bundle/manifest['jit']['compiler']),'--output',str(native),
               '--http-heap-bytes',str(manifest['jit']['heapBytes']),'--byte-array-limit',str(manifest['jit']['byteArrayLimit']),
               '--command-timeout','600']
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
                            'jit':[runtime,'run',str(assembly)],'aot':[str(native/'app')]}
        report['bundleManifest']=manifest
        report['loadBefore']=os.getloadavg()
        report['hostProcessesBefore']=subprocess.check_output(['ps','-axo','%cpu,comm'],text=True)
        modes=['interpreter','jit','aot']
        for index in range(-1,args.rounds):
            offset=max(index,0)%3
            for mode in modes[offset:]+modes[:offset]:
                row=workbench.server_sample(report['commands'][mode])
                row.update(mode=mode,round=index)
                report['warmups' if index<0 else 'samples'].append(row)
                (out/'results.json').write_text(json.dumps(report,indent=2)+'\n')
                print(f'{mode} round {index}: {row["requestsPerSecond"]:.2f} requests/s',flush=True)
        report['loadAfter']=os.getloadavg()
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
