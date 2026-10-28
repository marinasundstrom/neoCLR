"""Version-one GUI-neutral sessions from the existing owner GC snapshot protocol."""
import uuid

FORMAT = 'neoclr.diagnostics.session'
# These remain owner-thread counters/gauges, never process-wide aggregates.
METRICS = {
    'collections': ('count', 'counter'), 'allocations': ('count', 'counter'),
    'reclaimedAllocations': ('count', 'counter'), 'reclaimedPayloadBytes': ('bytes', 'counter'),
    'gcAttempts': ('count', 'counter'), 'gcFailures': ('count', 'counter'),
    'gcNanoseconds': ('ns', 'counter'), 'gcMaximumNanoseconds': ('ns', 'maximum'),
    'allocatedPayloadBytes': ('bytes', 'counter'),
    'livePayloadBytesAfterCollection': ('bytes', 'gauge'),
    'liveBlocksAfterCollection': ('count', 'gauge'),
    'peakLivePayloadBytesAfterCollection': ('bytes', 'maximum'),
    'peakArenaUsedBytes': ('bytes', 'maximum'),
}


def uint64(value):
    if type(value) is not int or not 0 <= value < 2**64:
        raise ValueError('expected unsigned 64-bit integer')
    return str(value)


def gc_session(rows):
    if not 1 <= len(rows) <= 10000:
        raise ValueError('capture must contain 1–10000 observations')
    identity = None
    observations = []
    last_sequence = last_timestamp = 0
    previous_metrics = None
    for row in rows:
        if (row.get('version') != 1 or row.get('capabilities') != ['gc-snapshot'] or
                row.get('scope') != 'runtime-owner-thread' or 'error' in row):
            raise ValueError('unsupported snapshot contract')
        current = (uint64(row['pid']), row['instance'])
        if not isinstance(current[1], str) or len(current[1]) != 32 or any(c not in '0123456789abcdef' for c in current[1]):
            raise ValueError('invalid runtime instance')
        if identity is not None and current != identity:
            raise ValueError('runtime instance changed')
        identity = current
        if type(row['available']) is not bool or type(row['fresh']) is not bool:
            raise ValueError('invalid availability')
        sequence = uint64(row['sequence'])
        stamp = uint64(row['timestampNanoseconds'])
        age = uint64(row['ageNanoseconds'])
        if row['pid'] == 0:
            raise ValueError('invalid process ID')
        if row['sequence'] == last_sequence and row['timestampNanoseconds'] != last_timestamp:
            raise ValueError('cached timestamp changed')
        if row['sequence'] < last_sequence or row['timestampNanoseconds'] < last_timestamp:
            raise ValueError('snapshot sequence/time regressed')
        if not row['available']:
            if row['fresh'] or row['sequence'] or row['timestampNanoseconds']:
                raise ValueError('invalid unavailable snapshot')
            metrics = None
            buckets = None
        else:
            if not row['sequence']:
                raise ValueError('available snapshot needs sequence')
            metrics = {name: uint64(row[name]) for name in METRICS}
            buckets = [uint64(v) for v in row['gcPauseBuckets']]
            if len(buckets) != 64:
                raise ValueError('expected 64 collection duration buckets')
            if row['sequence'] == last_sequence and previous_metrics != (metrics, buckets):
                raise ValueError('cached snapshot changed')
            if previous_metrics is not None:
                for name, (_, kind) in METRICS.items():
                    if kind != 'gauge' and int(metrics[name]) < int(previous_metrics[0][name]):
                        raise ValueError('counter reset during capture')
            previous_metrics = (metrics, buckets)
        observations.append(dict(source='owner', runtimeSequence=sequence,
                                 timestampNanoseconds=stamp if row['available'] else None,
                                 ageNanoseconds=age if row['available'] else None,
                                 availability='fresh' if row['fresh'] else 'cached' if row['available'] else 'unavailable',
                                 metrics=metrics, collectionDurationBuckets=buckets))
        last_sequence, last_timestamp = row['sequence'], row['timestampNanoseconds']
    return dict(format=FORMAT, version=1, sessionId=str(uuid.uuid4()),
                producer='neoclr-profiler',
                capabilities=dict(gcMetrics=True, executionTimeline=False, heapSnapshots=False),
                sources=[dict(id='owner', scope='runtime-owner-thread', pid=identity[0],
                              instance=identity[1], clock='monotonic-nanoseconds')],
                metricDefinitions={name: dict(unit=unit, kind=kind) for name, (unit, kind) in METRICS.items()},
                observations=observations,
                limitations=['Owner-thread snapshots at collection boundaries; not process totals.',
                             'Post-collection live payload is not instantaneous retained memory or RSS.',
                             'Collection duration is not universally stop-the-world pause time.',
                             'Cached observations do not establish zero activity.',
                             'No object graph, root path, generation or causal task data is captured.'])
