#!/usr/bin/env python3
"""Verify measured recovery, replay and error behavior, not abstract DST claims."""
import hashlib
import json
from pathlib import Path
import sys

root = Path(sys.argv[1])
report = {}
reference_trace = None
reference_rows = None
for backend in ['native', 'turmoil', 'commonware-external', 'madsim']:
    directory = root / f'{backend}-topology'
    assert (directory / 'exit-code.txt').read_text().strip() == '0', directory
    rows = [json.loads(line) for line in (directory / 'trials.jsonl').read_text().splitlines()]
    traces = sorted((directory / 'worker-traces').glob('trial-*.txt'))
    assert len(rows) == len(traces) == 10, directory
    for row, trace in zip(rows, traces):
        assert row['entries'] == 64 and row['recovered_equal'], row
        content = trace.read_bytes()
        assert len(content.splitlines()) == row['worker_events'] == 514, row
        if reference_trace is None:
            reference_trace = content
            reference_rows = rows
        assert content == reference_trace, f'Worker trace mismatch: {trace}'
    assert rows == reference_rows, directory
    failure_dir = root / f'{backend}-sync-failure'
    assert (failure_dir / 'exit-code.txt').read_text().strip() == '0', failure_dir
    failure = json.loads((failure_dir / 'trials.jsonl').read_text())
    assert failure['first_error'] == failure['idle_error'] == 'StorageFull', failure
    assert failure['later_error'] == 'Other' and failure['sync_calls'] == 1, failure
    # Producer event order is not controlled by the failure injector. Record,
    # rather than assert away, differences in that scenario's full event stream.
    failure_trace = (failure_dir / 'worker-traces/sync-failure.txt').read_bytes()
    report[backend] = {
        'trials': len(rows),
        'full_worker_trace_sha256': hashlib.sha256(reference_trace).hexdigest(),
        'topology_usage': json.loads((directory / 'usage.json').read_text()),
        'sync_failure': failure,
        'sync_failure_trace_sha256': hashlib.sha256(failure_trace).hexdigest(),
    }
(root / 'verified-comparison.json').write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps(report, indent=2))
