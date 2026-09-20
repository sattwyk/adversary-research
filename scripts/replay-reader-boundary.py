#!/usr/bin/env python3
"""Replay the recorded Shuttle schedule against the actual historical reader."""
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time

checkout, results = map(Path, sys.argv[1:3])
original = (results / 'parent-level-2.log').read_text()
schedules = re.findall(r'failing schedule:\s*"\s*([0-9a-f]+)\s*"', original)
assert schedules, 'No real recorded schedule to replay'
schedule = schedules[-1]
(results / 'reader-schedule.txt').write_text(schedule + '\n')
env = dict(os.environ, READER_LEVEL='2', EXPECT_RACE='1', READER_REPLAY=schedule,
           CARGO_TARGET_DIR='/tmp/adversary-reader-target', CARGO_PROFILE_DEV_DEBUG='0')
records = []
for trial in range(10):
    started = time.perf_counter()
    run = subprocess.run(['cargo', '+nightly-2025-12-11', 'test', '-j', '2', '--lib',
                          'adversary_reader_boundary', '--', '--nocapture'],
                         cwd=checkout, env=env, capture_output=True, text=True, timeout=60)
    output = run.stdout + run.stderr
    (results / f'parent-replay-{trial}.log').write_text(output)
    assert run.returncode == 0, output
    summary = json.loads(re.search(r'READER_SUMMARY (\{.*\})', output)[1])
    assert summary['found'] and summary['executions'] == 1, summary
    records.append(dict(trial=trial, wall_including_cargo_seconds=time.perf_counter()-started, **summary))
(results / 'replay.json').write_text(json.dumps(records, indent=2) + '\n')
