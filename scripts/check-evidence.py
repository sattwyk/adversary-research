#!/usr/bin/env python3
"""Check repository navigation, immutable archives and recorded trial integrity.

This checks packaged evidence, not production correctness. Run the Rust probes
for fresh execution evidence. Python's standard library is sufficient.
"""
from pathlib import Path
import hashlib
import json
import re
import subprocess
from urllib.parse import unquote, urlsplit

ROOT = Path(__file__).resolve().parents[1]


def main():
    for script in sorted((ROOT / "scripts").glob("*.sh")):
        subprocess.run(["bash", "-n", str(script)], check=True)
    for line in (ROOT / "archive/SHA256SUMS").read_text().splitlines():
        expected, name = line.split("  ", 1)
        actual = hashlib.sha256((ROOT / name).read_bytes()).hexdigest()
        assert actual == expected, f"Archive checksum mismatch: {name}"

    # Historical machine-specific scripts/README are intentionally archived as-is.
    # Maintained documentation and exported analysis should navigate in GitHub.
    files = [ROOT / "README.md", ROOT / "THIRD_PARTY.md"]
    files += list((ROOT / "docs").rglob("*.md"))
    files += list((ROOT / "experiments").rglob("*.md"))
    pattern = re.compile(r"\[[^\]]*\]\((<[^>]+>|[^)\s]+)(?:\s+[^)]*)?\)")
    for path in files:
        for match in pattern.finditer(path.read_text()):
            target = match[1].strip("<>")
            if urlsplit(target).scheme or target.startswith("#"):
                continue
            target = unquote(target.split("#", 1)[0])
            assert (path.parent / target).exists(), f"Broken local link: {path}: {target}"

    trial_file = ROOT / "results/2026-09-19/native-topology/trials.jsonl"
    trials = [json.loads(line) for line in trial_file.read_text().splitlines()]
    assert len(trials) == 20, "Expected the 20 recorded native topology trials"
    assert [trial["trial"] for trial in trials] == list(range(20))
    for trial in trials:
        assert trial["recovered_equal"] is True
        assert trial["entries"] == 256
        assert trial["abandoned_waiters"] == 24
        assert trial["batches"], "An empty logger result is not batching evidence"
    distinct = len({tuple(trial["batches"]) for trial in trials})
    controlled_path = ROOT / "results/2026-09-20/controlled-topology/trials.jsonl"
    controlled = [json.loads(line) for line in controlled_path.read_text().splitlines()]
    assert len(controlled) == 10
    for index, trial in enumerate(controlled):
        assert trial["trial"] == index
        assert trial["recovered_equal"] and trial["entries"] == 64
        assert trial["worker_events"] == 514
        assert trial["short_write_calls"] == 1988
        assert trial["sync_calls"] == 26
        assert trial["batches"] == controlled[0]["batches"]
    failure_path = ROOT / "results/2026-09-20/controlled-sync-failure-final/trials.jsonl"
    failure = json.loads(failure_path.read_text())
    assert failure["first_error"] == failure["idle_error"] == "StorageFull"
    assert failure["later_error"] == "Other" and failure["sync_calls"] == 1
    history_path = ROOT / "results/2026-09-20/historical-state-final/state.jsonl"
    history = [json.loads(line) for line in history_path.read_text().splitlines()]
    assert len(history) == 6
    assert [record["rejected"] for record in history] == [False, True] * 3
    print("PASS: ten controlled replay records, sync failure, six historical state records.")
    print(f"PASS: shell syntax, archive hashes, local documentation links, 20 recorded native trials ({distinct} distinct batch sequences).")


if __name__ == "__main__":
    main()
