#!/usr/bin/env python3
"""Retain complete reports compressed, with small browsable summaries alongside."""
import argparse
import gzip
import hashlib
import json
from pathlib import Path


def summarize(value):
    if isinstance(value, list):
        return [summarize(item) for item in value]
    if not isinstance(value, dict):
        return value
    result = {}
    for key, item in value.items():
        if key == 'requests' and isinstance(item, list):
            result['rawRequestCount'] = len(item)
        elif key == 'allocationSites':
            result['allocationSitesInRawReport'] = True
        elif key == 'stdout':
            result['stdoutInRawReport'] = True
        else:
            result[key] = summarize(item)
    return result


def archive(source, destination):
    raw = source.read_bytes()
    report = json.loads(raw)
    packed = destination.with_suffix('.json.gz')
    packed.write_bytes(gzip.compress(raw, mtime=0))
    result = summarize(report)
    result['rawReport'] = dict(file=packed.name, sha256=hashlib.sha256(raw).hexdigest(),
                               compressedSha256=hashlib.sha256(packed.read_bytes()).hexdigest())
    destination.write_text(json.dumps(result, indent=2)+'\n')


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('source', type=Path)
    p.add_argument('destination', type=Path)
    a = p.parse_args()
    archive(a.source, a.destination)
