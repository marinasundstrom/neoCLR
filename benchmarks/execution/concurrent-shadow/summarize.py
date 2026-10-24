"""Validate and summarize diagnostic shadow cohorts; these are not capacity runs."""
import argparse
import gzip
import json
from pathlib import Path


def summarize(paths):
    result = []
    seen = set()
    for path in paths:
        report = json.loads(gzip.decompress(path.read_bytes()))
        assert report['passed'] and len(report['trials']) == 2
        for row in report['trials']:
            key = report['sample'], row['mode']
            assert key not in seen
            seen.add(key)
            count, copied, capture, cpu, invalid = row['shadow']
            assert row['exitCode'] == 0 and row['count'] == 1024
            assert count == row['gc'][0] and not invalid and not row['gc'][1]
            assert not sum(row['failures']) and sum(row['requests']) == count
            result.append(dict(sample=key[0], mode=key[1], requests=row['count'],
                comparedCollections=count, copiedBytesPerRequest=copied/row['count'],
                captureMicrosecondsPerRequest=capture/row['count']/1000,
                backgroundCpuMicrosecondsPerRequest=cpu/row['count']/1000))
    assert seen == {('plain','aot'), ('plain','jit'), ('json','aot'), ('json','jit')}
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('inputs', nargs=2, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    args.output.write_text(json.dumps(summarize(args.inputs), indent=2) + '\n')
