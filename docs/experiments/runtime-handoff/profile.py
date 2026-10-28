#!/usr/bin/env python3
"""Separate macOS thread samples of each validated offload implementation."""
import argparse
import json
import os
from pathlib import Path
import select
import socket
import subprocess
import time

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('--report', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
a.output.mkdir(parents=True, exist_ok=False)
source = json.loads(a.report.read_text()); assert source['passed']
env = dict(os.environ, **source['settings']['dotnetOverrides'])
report = []
try:
    for runtime in ('neo-sync','neo-concurrent','go','dotnet'):
        row = next(s for s in source['samples'] if s['runtime']==runtime and s['mode']=='offload')
        command = list(row['command']); command[-2] = '1024'
        process = subprocess.Popen(command, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, env=env)
        capture = None
        try:
            assert select.select([process.stdout], [], [], 10)[0]
            port = int(process.stdout.readline())
            sample_command = ['sample',str(process.pid),'2','1','-file',str((a.output/(runtime+'.sample.txt')).resolve())]
            capture = subprocess.Popen(sample_command, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            with socket.create_connection(('127.0.0.1',port),timeout=10) as peer:
                peer.setsockopt(socket.IPPROTO_TCP,socket.TCP_NODELAY,1)
                for _ in range(1024):
                    assert peer.recv(1)==b'R'; time.sleep(.0002)
                    peer.sendall(b'x'); assert peer.recv(1)==b'A'
            stdout,stderr = process.communicate(timeout=10)
            sample_out,sample_err = capture.communicate(timeout=10)
            row = dict(runtime=runtime,command=command,exitCode=process.returncode,stdout=stdout,stderr=stderr,
                       sampleCommand=sample_command,sampleExitCode=capture.returncode,sampleStdout=sample_out,sampleStderr=sample_err)
            report.append(row)
            assert process.returncode==0 and capture.returncode==0,row
            assert json.loads(stdout)['checksum']==source['expectedChecksum']
        finally:
            for child in (process,capture):
                if child is not None and child.poll() is None:
                    child.kill(); child.communicate()
finally:
    (a.output/'report.json').write_text(json.dumps(report,indent=2)+'\n')
print('PASS four separate runtime offload profiles; timings excluded')
