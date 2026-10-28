#!/usr/bin/env python3
"""Owner-affine TCP acknowledgement with inline or independently offloaded CPU work."""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import select
import shutil
import socket
import statistics
import subprocess
import time

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--output', type=Path, required=True)
p.add_argument('--rounds', type=int, default=256)
p.add_argument('--pairs', type=int, default=3)
p.add_argument('--iterations', type=int, default=3200000)
a = p.parse_args()
assert 1 <= a.rounds <= 4000 and 1 <= a.pairs <= 10 and 1 <= a.iterations <= 100000000
out = a.output.resolve(); out.mkdir(parents=True, exist_ok=False)
here = Path(__file__).resolve().parent; root = here.parents[2]
native = root / 'runtime/native'; legacy = root / 'docs/experiments/aot-console'; live = native / 'gc/concurrent'
env = dict(os.environ, DOTNET_TieredCompilation='0', DOTNET_GCServer='0', DOTNET_GCConcurrent='1')
report = dict(passed=False, platform=platform.platform(), commands=[], checks=[], samples=[],
              settings=dict(rounds=a.rounds, warmup=32, pairs=a.pairs, iterations=a.iterations,
                            peerDelaySeconds=0.0002, dotnetOverrides={k: env[k] for k in ('DOTNET_TieredCompilation','DOTNET_GCServer','DOTNET_GCConcurrent')}))


def run(command, cwd=out):
    command = list(map(str, command))
    r = subprocess.run(command, cwd=cwd, env=env, text=True, capture_output=True, timeout=60)
    report['commands'].append(dict(command=command, cwd=str(cwd), exitCode=r.returncode, stdout=r.stdout, stderr=r.stderr))
    r.check_returncode(); return r.stdout


def expected(iterations):
    x = 1; mask = (1 << 64) - 1
    for _ in range(iterations):
        x ^= (x << 13) & mask; x ^= x >> 7; x ^= (x << 17) & mask
    return str(x)


def measure(command, mode, rounds, warmup, checksum):
    cmd = [*map(str, command), mode, str(rounds + warmup), str(a.iterations)]
    process = subprocess.Popen(cmd, cwd=out, env=env, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    values = []; delays = []
    try:
        assert select.select([process.stdout], [], [], 10)[0], 'port timeout'
        line = process.stdout.readline()
        assert line.strip().isdigit(), (cmd, line, process.poll())
        with socket.create_connection(('127.0.0.1', int(line)), timeout=10) as peer:
            peer.setsockopt(socket.IPPROTO_TCP, socket.TCP_NODELAY, 1)
            begin = time.perf_counter_ns()
            for round_id in range(rounds + warmup):
                assert peer.recv(1) == b'R', 'readiness protocol'
                delay_start = time.perf_counter_ns()
                time.sleep(0.0002)
                sent = time.perf_counter_ns()
                peer.sendall(b'x')
                assert peer.recv(1) == b'A', 'acknowledgement protocol'
                ended = time.perf_counter_ns()
                if round_id >= warmup:
                    values.append(ended - sent); delays.append(sent - delay_start)
            wall = (time.perf_counter_ns() - begin) / 1e9
        stdout, stderr = process.communicate(timeout=10)
        assert process.returncode == 0, (cmd, process.returncode, stderr)
        server = json.loads(stdout)
        assert server['checksum'] == ('0' if mode == 'idle' else checksum), server
        assert server['rounds'] == rounds + warmup and server['iterations'] == a.iterations
        ordered = sorted(values)
        return dict(command=cmd, server=server, stderr=stderr, peerWallSeconds=wall,
                    ackP50ns=statistics.median(values), ackP99ns=ordered[math.ceil(len(values)*.99)-1],
                    ackMaxNs=max(values), latencyNs=values, observedPeerDelayNs=delays)
    except BaseException as error:
        if process.poll() is None:
            process.kill()
        stdout, stderr = process.communicate()
        report.setdefault('failures', []).append(dict(command=cmd, error=str(error),
            exitCode=process.returncode, stdout=stdout, stderr=stderr))
        raise
    finally:
        if process.poll() is None:
            process.kill(); process.communicate()


try:
    report['revision'] = run(['git', 'rev-parse', 'HEAD'], root).strip()
    report['toolchains'] = {'go':run(['go','version']).strip(), 'dotnetRuntimes':run(['dotnet','--list-runtimes'])}
    cc = [run(['xcrun','--find','clang']).strip(), '-isysroot', run(['xcrun','--show-sdk-path']).strip()]
    flags = ['-std=c11','-O2','-Wall','-Wextra','-Werror','-Wno-deprecated-declarations','-pthread','-Wl,-dead_strip',
             '-I',native/'async','-I',native/'gc','-I',native/'workers','-I',legacy]
    run(['python3',live/'generate.py','--output',out/'live-gc.c'])
    policy = json.loads((live/'policy.json').read_text())
    common = [here/'neo.c',native/'async/native-runtime.c',native/'workers/native-workers.c',native/'gc/root-probe.c',
              legacy/'native-stack.c',legacy/'socket-listener.c',legacy/'text-arena.c']
    binaries = {}
    checksum = expected(a.iterations); report['expectedChecksum'] = checksum
    for mode in ('sync','concurrent'):
        defs = ['-DNEOCLR_NATIVE_GC'] if mode == 'sync' else ['-D'+v for v in policy['defines']] + ['-DTRACE_LIVE','-I',live]
        gc = [native/'gc/native-gc.c'] if mode == 'sync' else [out/'live-gc.c',live/'gate.c']
        for variant, extra in [('optimized',[]),('ubsan',['-fsanitize=undefined','-fno-sanitize-recover=all'])]:
            binary = out/f'neo-{mode}-{variant}'
            run([*cc,*flags,*defs,*extra,*common,*gc,'-o',binary])
            for shape in ('idle','inline','offload'):
                check = measure([binary],shape,8,0,checksum)
                check.update(runtime='neo-'+mode, variant=variant, mode=shape)
                report['checks'].append(check)
        binaries['neo-'+mode] = [out/f'neo-{mode}-optimized']
    run(['go','build','-o',out/'go-probe',here/'go.go'])
    binaries['go'] = [out/'go-probe']
    build = out/'dotnet'; shutil.copytree(here/'dotnet',build)
    (build/'global.json').write_text('{"sdk":{"version":"10.0.100","rollForward":"disable"}}\n')
    report['toolchains']['dotnetSdk'] = run(['dotnet','--version'],build).strip()
    run(['dotnet','build','-c','Release','-o',out/'dotnet-bin',build/'Probe.csproj'],build)
    binaries['dotnet'] = ['dotnet',out/'dotnet-bin/Probe.dll']
    for runtime in ('go','dotnet'):
        for mode in ('idle','inline','offload'):
            check = measure(binaries[runtime],mode,8,0,checksum)
            check.update(runtime=runtime,variant='optimized',mode=mode); report['checks'].append(check)
    # Rotate runtime order, reverse mode order on alternating repetitions.
    names = list(binaries)
    for pair in range(a.pairs):
        for mode in (('idle','inline','offload') if pair%2==0 else ('offload','inline','idle')):
            for runtime in names[pair%4:] + names[:pair%4]:
                sample = measure(binaries[runtime],mode,a.rounds,32,checksum)
                sample.update(runtime=runtime,mode=mode,pair=pair); report['samples'].append(sample)
    paths = [*common,Path(__file__),here/'go.go',*list((here/'dotnet').glob('*')),out/'live-gc.c']
    paths += [f for f in native.rglob('*') if f.suffix in ('.c','.h','.json','.py')]
    paths += list(legacy.glob('socket-*.h'))
    report['inputs'] = {str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in paths if p.is_file()}
    report['binaries'] = {name:hashlib.sha256(Path(cmd[-1]).read_bytes()).hexdigest() for name,cmd in binaries.items()}
    report['passed'] = True
finally:
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n')
print('PASS owner-affine TCP/CPU comparison across neoCLR collectors, Go and .NET')
