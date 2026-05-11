from __future__ import annotations
import platform, sys, json, statistics
from pathlib import Path
from typing import Any

def percentile(samples: list[float], pct: float) -> float:
    if not samples: return 0.0
    ordered = sorted(samples)
    if len(ordered) == 1: return ordered[0]
    rank = (len(ordered) - 1) * pct / 100.0
    lo = int(rank); hi = min(lo + 1, len(ordered) - 1); frac = rank - lo
    return ordered[lo] * (1.0 - frac) + ordered[hi] * frac

def proof_status(measured_qps: float, p99_ms: float, error_rate: float, valid: bool, target_qps: float = 10_000.0, p99_threshold_ms: float = 5.0) -> str:
    if not valid: return "invalid"
    if measured_qps >= target_qps and p99_ms < p99_threshold_ms and error_rate == 0.0: return "proven"
    return "not_proven"

def environment_metadata(mode: str = "local") -> dict[str, Any]:
    return {"os": platform.platform(), "cpu": platform.processor() or platform.machine(), "python": sys.version.split()[0], "mode": mode}

def write_report(path: str | Path, report: dict[str, Any]) -> None:
    Path(path).write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
