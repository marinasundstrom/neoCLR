#!/usr/bin/env python3
import json
import statistics
import sys


def summarize(report):
    if not report.get('passed') or report.get('profile'):
        raise ValueError('expected successful unprofiled cohort')
    result={}
    if [r['label'] for r in report['trials']]!=['system','arena','arena','system']:
        raise ValueError('unexpected cohort')
    for label in ['system','arena']:
        rows=[r for r in report['trials'] if r['label']==label]
        if any(r['exitCode'] or r['jit']['guestExitCode'] for r in rows):
            raise ValueError('failed invocation')
        rates=[r['trial']['requestsPerSecond'] for r in rows]
        result[label]=dict(requestsPerSecond=statistics.mean(rates),rateRange=[min(rates),max(rates)],
                           cpuMicrosecondsPerResponse=statistics.mean(r['cpuMicrosecondsPerResponse'] for r in rows))
    result['throughputGainPercent']=100*(result['arena']['requestsPerSecond']/result['system']['requestsPerSecond']-1)
    result['cpuReductionPercent']=100*(1-result['arena']['cpuMicrosecondsPerResponse']/result['system']['cpuMicrosecondsPerResponse'])
    return result

if __name__=='__main__':print(json.dumps(summarize(json.load(open(sys.argv[1]))),indent=2))
