#!/usr/bin/env python3
"""Measure one isolated Linux child; compilation is outside the timed interval."""
import json
import os
from pathlib import Path
import resource
import subprocess
import sys
import time

directory = Path(sys.argv[1])
directory.mkdir(parents=True, exist_ok=True)
started = time.perf_counter()
timed_out = False
with (directory / "trials.jsonl").open("w") as out, (directory / "run.log").open("w") as err:
    try:
        result = subprocess.run(sys.argv[2:], stdout=out, stderr=err,
                                timeout=float(os.environ.get("ADVERSARY_PROCESS_TIMEOUT", "60")))
        returncode = result.returncode
    except subprocess.TimeoutExpired:
        timed_out = True
        returncode = 124
usage = resource.getrusage(resource.RUSAGE_CHILDREN)
metrics = {
    "wall_seconds": time.perf_counter() - started,
    "user_cpu_seconds": usage.ru_utime,
    "system_cpu_seconds": usage.ru_stime,
    "peak_rss_kib": usage.ru_maxrss,
    "exit_code": returncode,
    "external_timeout": timed_out,
}
(directory / "usage.json").write_text(json.dumps(metrics, indent=2) + "\n")
(directory / "exit-code.txt").write_text(str(returncode) + "\n")
print(json.dumps(metrics))
sys.exit(returncode if returncode >= 0 else 128 - returncode)
