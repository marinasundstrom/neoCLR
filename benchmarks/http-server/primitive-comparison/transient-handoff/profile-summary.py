#!/usr/bin/env python3
"""Validate matched diagnostic reports and normalize incremental root work."""
import argparse
import gzip
import hashlib
import json
from pathlib import Path


def summarize(small, large):
    assert small['passed'] and large['passed']
    assert small['paced'] and large['paced']
    assert small['json'] == large['json']
    def identities(report):
        return {Path(path).name if path.endswith(('.o', 'diagnostic-probe.c', 'diagnostic-gc.c', 'profile-count.py')) else path: digest
                for path, digest in report['inputs'].items()}
    assert identities(small) == identities(large), 'configuration changed'
    added = len(large['responses']) - len(small['responses'])
    assert added > 0
    result = {key: (large['detail'][key] - value) / added
              for key, value in small['detail'].items()}
    assert all(value >= 0 for value in result.values())
    result['boundaries'] = (large['boundaries'] - small['boundaries']) / added
    assert result['boundaries'] > 0
    result['framesPerBoundary'] = result['ancestorFrames'] / result['boundaries']
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('reports', type=Path, nargs=4, help='plain small/large, JSON small/large')
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    reports = [json.loads(gzip.decompress(p.read_bytes()) if p.suffix == '.gz' else p.read_bytes()) for p in args.reports]
    assert not reports[0]['json'] and reports[2]['json']
    for leaf in ['diagnostic-probe.c', 'diagnostic-gc.c', 'profile-count.py']:
        hashes = [next(d for p, d in r['inputs'].items() if p.endswith('/' + leaf)) for r in reports]
        assert len(set(hashes)) == 1
    native = [{p: d for p, d in r['inputs'].items() if '/runtime/native/' in p} for r in reports]
    assert all(n == native[0] for n in native)
    result = dict(unit='additional checked HTTP response',
                  plain=summarize(*reports[:2]), json=summarize(*reports[2:]),
                  inputs={str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in args.reports})
    args.output.write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result, indent=2))
