#!/usr/bin/env python3
"""Capture separate macOS stacks; never mix sampled runs into the timing table."""
import argparse
import json
from pathlib import Path
import subprocess

p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--binaries',type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
a=p.parse_args();a.output.mkdir(parents=True,exist_ok=False)
report=[]
try:
    for mode in ('sync','concurrent'):
        for wake in (0,1):
            name=f'{mode}-{wake}'
            command=[str((a.binaries/f'{mode}-optimized').resolve()),str(wake),'2048']
            with (a.output/(name+'.json')).open('w') as stream:
                process=subprocess.Popen(command,stdout=stream,stderr=subprocess.PIPE,text=True)
                try:
                    sample_command=['sample',str(process.pid),'2','1','-file',str((a.output/(name+'.sample.txt')).resolve())]
                    capture=subprocess.run(sample_command,capture_output=True,text=True,timeout=30)
                    _,error=process.communicate(timeout=60)
                    report.append(dict(command=command,exitCode=process.returncode,stderr=error,
                                       sampleCommand=sample_command,sampleExitCode=capture.returncode,
                                       sampleStdout=capture.stdout,sampleStderr=capture.stderr))
                    assert process.returncode==0 and capture.returncode==0,report[-1]
                finally:
                    if process.poll() is None: process.kill();process.communicate()
finally:
    (a.output/'report.json').write_text(json.dumps(report,indent=2)+'\n')
print('PASS separate polling/wake profiles for both collectors')
