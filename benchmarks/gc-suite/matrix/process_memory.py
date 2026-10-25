"""macOS process snapshots, not allocator accounting or peak-memory estimates."""
import subprocess


def parse_snapshot(text, pid):
    rows = text.splitlines()
    if len(rows) != 1:
        raise ValueError('expected one owned process row')
    fields = rows[0].split()
    if len(fields) != 3:
        raise ValueError('expected pid, RSS and virtual size')
    observed_pid, rss_kib, virtual_kib = map(int, fields)
    if observed_pid != pid or rss_kib < 0 or virtual_kib < 0:
        raise ValueError('invalid process identity or memory counters')
    return dict(pid=pid, rssBytes=rss_kib * 1024, virtualBytes=virtual_kib * 1024)


def snapshot(pid):
    command = ['/bin/ps', '-p', str(pid), '-o', 'pid=,rss=,vsz=']
    result = subprocess.run(command, capture_output=True, text=True, timeout=5, check=True)
    return dict(parse_snapshot(result.stdout, pid), command=command,
                stdout=result.stdout, stderr=result.stderr, exitCode=result.returncode)
