from __future__ import annotations
import argparse, asyncio, time, sys
from typing import Any
import httpx
from .report import environment_metadata, percentile, proof_status, write_report

async def run_benchmark(server_url: str, scenario: str, duration: float, warmup: float, concurrency: int, target_qps: float, users: int, items: int, contexts: int, embedding_dims: int, seed: int, mode: str = "local", command: str | None = None) -> dict[str, Any]:
    valid = duration >= 1.0 and warmup >= 0 and concurrency > 0
    notes: list[str] = []
    if duration < 1.0: notes.append("duration below 1 second is invalid")
    latencies: list[float] = []
    errors = 0
    completed = 0
    entity_lookups = 0
    stop = time.perf_counter() + warmup + duration
    measure_start = time.perf_counter() + warmup
    timeout = httpx.Timeout(5.0)
    sem = asyncio.Semaphore(concurrency)

    async with httpx.AsyncClient(base_url=server_url.rstrip('/'), timeout=timeout) as client:
        async def one_request(i: int) -> None:
            nonlocal errors, completed, entity_lookups
            async with sem:
                path, payload, lookups = _scenario_request(scenario, i, users, items, contexts)
                t0 = time.perf_counter()
                try:
                    if payload is None:
                        response = await client.get(path)
                    else:
                        response = await client.post(path, json=payload)
                    ok = response.status_code // 100 == 2
                    if not ok: errors += 1
                except Exception:
                    ok = False; errors += 1
                elapsed_ms = (time.perf_counter() - t0) * 1000.0
                if time.perf_counter() >= measure_start:
                    completed += 1
                    entity_lookups += lookups
                    if ok: latencies.append(elapsed_ms)
        i = 0
        tasks: set[asyncio.Task[None]] = set()
        while time.perf_counter() < stop:
            while len(tasks) < concurrency and time.perf_counter() < stop:
                task = asyncio.create_task(one_request(i)); tasks.add(task); i += 1
            if tasks:
                done, tasks = await asyncio.wait(tasks, timeout=0.001, return_when=asyncio.FIRST_COMPLETED)
            if target_qps > 0:
                await asyncio.sleep(min(0.001, 1.0 / target_qps))
        if tasks: await asyncio.gather(*tasks, return_exceptions=True)
    measured_qps = completed / duration if duration > 0 else 0.0
    error_rate = errors / max(completed + errors, 1)
    p99 = percentile(latencies, 99)
    status = proof_status(measured_qps, p99, error_rate, valid, target_qps=target_qps)
    return {
        "status": status, "scenario": scenario, "server_url": server_url, "command": command or " ".join(sys.argv),
        "duration_seconds": duration, "warmup_seconds": warmup, "target_qps": target_qps, "measured_qps": measured_qps,
        "entity_lookups_per_second": entity_lookups / duration if duration > 0 else 0.0,
        "latency_ms": {"p50": percentile(latencies, 50), "p95": percentile(latencies, 95), "p99": p99, "max": max(latencies) if latencies else 0.0},
        "errors": {"count": errors, "rate": error_rate},
        "dataset": {"users": users, "items": items, "contexts": contexts, "embedding_dims": embedding_dims, "seed": seed},
        "environment": environment_metadata(mode), "proof_threshold": {"qps": target_qps, "p99_ms": 5.0}, "notes": notes,
    }

def _scenario_request(scenario: str, i: int, users: int, items: int, contexts: int) -> tuple[str, dict[str, Any] | None, int]:
    if scenario == "single_get_request_qps":
        return f"/v1/features/user/{i % max(users, 1)}?features=age_bucket,country_id", None, 1
    if scenario == "batch_post_request_qps":
        reqs = [{"entity": "user", "id": str((i * 8 + j) % max(users, 1)), "features": ["age_bucket", "country_id"]} for j in range(8)]
        return "/v1/features/lookup", {"requests": reqs, "include_missing": True}, len(reqs)
    if scenario == "recommendation_triple_lookup":
        reqs = [{"entity": "user", "id": str(i % max(users, 1)), "features": ["age_bucket"]}, {"entity": "item", "id": str(i % max(items, 1)), "features": ["category_id"]}, {"entity": "context", "id": str(i % max(contexts, 1)), "features": ["hour_bucket"]}]
        return "/v1/features/lookup", {"requests": reqs, "include_missing": True}, 3
    raise ValueError(f"unknown scenario: {scenario}")

def benchmark_main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Run Featherstore benchmark and write an honest JSON report")
    parser.add_argument("--server", default="http://localhost:8080"); parser.add_argument("--scenario", default="single_get_request_qps", choices=["single_get_request_qps", "batch_post_request_qps", "recommendation_triple_lookup"])
    parser.add_argument("--duration", type=float, default=30); parser.add_argument("--warmup", type=float, default=5); parser.add_argument("--concurrency", type=int, default=128); parser.add_argument("--target-qps", type=float, default=10000)
    parser.add_argument("--users", type=int, default=100000); parser.add_argument("--items", type=int, default=100000); parser.add_argument("--contexts", type=int, default=24); parser.add_argument("--embedding-dims", type=int, default=8); parser.add_argument("--seed", type=int, default=13)
    parser.add_argument("--mode", default="local"); parser.add_argument("--output", default="benchmark-report.json")
    args = parser.parse_args(argv)
    report = asyncio.run(run_benchmark(args.server, args.scenario, args.duration, args.warmup, args.concurrency, args.target_qps, args.users, args.items, args.contexts, args.embedding_dims, args.seed, args.mode, command=" ".join(sys.argv)))
    write_report(args.output, report)
    print(f"wrote {args.output}: status={report['status']} measured_qps={report['measured_qps']:.2f} p99_ms={report['latency_ms']['p99']:.3f} errors={report['errors']['count']}")
    return 2 if report["status"] == "invalid" else 0

if __name__ == "__main__": raise SystemExit(benchmark_main())
