#!/usr/bin/env python3
"""Capture separate macOS thread-stack samples; never count these as timings."""
import argparse
import json
from pathlib import Path
import subprocess
p=argparse.ArgumentParser(description=__doc__);p.add_argument('--binaries',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();a.output.mkdir(parents=True,exist_ok=False)
report=[]
for policy in ('fifo','local'):
 result=a.output/(policy+'.json');sample=a.output/(policy+'.sample.txt')
 with result.open('w') as stream:
  command=[str((a.binaries/(policy+'-benchmark')).resolve()),'4','1','262144']
  process=subprocess.Popen(command,stdout=stream,stderr=subprocess.PIPE,text=True)
  capture=subprocess.run(['sample',str(process.pid),'2','1','-file',str(sample.resolve())],capture_output=True,text=True,timeout=30)
  _,error=process.communicate(timeout=60)
  report.append(dict(policy=policy,command=command,exitCode=process.returncode,stderr=error,sampleExitCode=capture.returncode,sampleStdout=capture.stdout,sampleStderr=capture.stderr))
  assert process.returncode==0 and capture.returncode==0,report[-1]
(a.output/'report.json').write_text(json.dumps(report,indent=2)+'\n')
print('Captured separate 4-worker nested-tiny thread samples')
