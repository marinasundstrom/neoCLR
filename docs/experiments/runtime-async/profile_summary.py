"""Bounded GC diagnostics: histogram quantiles are upper bounds, not exact pauses."""


def pause_summary(samples):
    buckets = [0] * 64
    for sample in samples:
        row = sample['gcPauseBuckets']
        if len(row) != 64 or any(type(n) is not int or n < 0 for n in row):
            raise ValueError('invalid GC pause histogram')
        if sum(row) != sample['gcAttempts'] or sample['gcFailures']:
            raise ValueError('GC attempts/failures disagree with a successful profile')
        if (sample['livePayloadBytesAfterCollection'] or sample['liveBlocksAfterCollection']
                or sample['allocatedPayloadBytes'] != sample['reclaimedPayloadBytes']):
            raise ValueError('unreclaimed payload in GC profile')
        buckets = [a + b for a, b in zip(buckets, row)]
    total = sum(buckets)
    if not total:
        raise ValueError('no collection attempts to summarize')

    def upper_bound(percent):
        target = (total * percent + 99) // 100
        cumulative = 0
        for index, count in enumerate(buckets):
            cumulative += count
            if cumulative >= target:
                return (1 << (index + 1)) - 1
        raise AssertionError('unreachable quantile')

    return dict(attempts=total, buckets=buckets, bucketUnit='nanoseconds',
                bucketDefinition='i: [2^i, 2^(i+1)-1]; bucket 0 also includes zero',
                p50UpperBoundNanoseconds=upper_bound(50),
                p95UpperBoundNanoseconds=upper_bound(95),
                p99UpperBoundNanoseconds=upper_bound(99),
                maximumNanoseconds=max(s['gcMaximumNanoseconds'] for s in samples))
