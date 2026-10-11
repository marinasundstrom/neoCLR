"""Strict, versioned settings for the currently supported external measurements."""
import math

DEFAULTS = dict(measurement='batch-throughput', modes=['interpreter', 'jit', 'aot'],
                rounds=3, warmups=2, operations=8, timeout=120)
MEASUREMENTS = {
    'startup': ('startupSeconds', 'seconds', 'Process launch to READY, including runtime loading, JIT and workload setup'),
    'batch-latency': ('batchSeconds', 'seconds', 'BEGIN to END, including workload validation, callback and transport costs'),
    'batch-throughput': ('operationsPerSecond', 'operations/second', 'Validated operations per external BEGIN-to-END interval'),
    'first-invocation': ('batchSeconds', 'seconds', 'First worker invocation after setup, including validation and transport; setup may already exercise the workload'),
}


def resolve(manifest=None, external=None, overrides=None):
    effective = dict(DEFAULTS)
    origins = {name: 'default' for name in effective}
    for origin, settings in [('manifest', manifest), ('config', external), ('override', overrides)]:
        if settings is None:
            continue
        if not isinstance(settings, dict):
            raise ValueError(origin+' benchmark configuration must be an object')
        unknown = set(settings) - set(DEFAULTS)
        if unknown:
            raise ValueError('Unsupported benchmark settings: '+', '.join(sorted(unknown)))
        effective.update(settings)
        origins.update({name: origin for name in settings})
    measurement = effective['measurement']
    if not isinstance(measurement, str) or measurement not in MEASUREMENTS:
        raise ValueError('Unsupported measurement; internal timing, allocations and GC diagnostics are not implemented')
    modes = effective['modes']
    if not isinstance(modes, list) or not modes or any(not isinstance(m, str) or m not in DEFAULTS['modes'] for m in modes) or len(set(modes)) != len(modes):
        raise ValueError('Modes must be a nonempty unique list of interpreter, jit, aot')
    for key, lower, upper in [('rounds', 3, 10000), ('warmups', 0, 10000), ('operations', 1, 10000)]:
        value = effective[key]
        if type(value) is not int or not lower <= value <= upper:
            raise ValueError(f'{key} must be an integer in {lower}..{upper}')
    timeout = effective['timeout']
    if type(timeout) not in (int, float) or not math.isfinite(timeout) or not 0 < timeout <= 3600:
        raise ValueError('timeout must be a finite number in (0, 3600] seconds')
    if measurement == 'first-invocation' and (effective['warmups'] != 0 or effective['operations'] != 1):
        raise ValueError('first-invocation requires warmups=0 and operations=1')
    return effective, origins
